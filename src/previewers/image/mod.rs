mod backend;

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Mutex,
        mpsc::{self, Receiver, TryRecvError},
    },
    thread,
    time::SystemTime,
};

use ::image::{DynamicImage, ImageReader};
use anyhow::Result;
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Clear, Paragraph, Wrap},
};
use ratatui_image::{StatefulImage, protocol::StatefulProtocol};

const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "bmp", "ico", "webp", "tiff", "tif", "tga", "dds", "hdr", "exr",
    "farbfeld", "pam", "pbm", "pgm", "pnm", "ppm", "avif", "heic", "heif", "qoi",
];

/// Probe the running terminal for graphics capabilities so image previews can
/// reuse the detected protocol.
pub fn initialize_graphics() -> Result<()> {
    backend::initialize()
}

/// Renders image previews by decoding files in the background and delegating to
/// `ratatui-image` for terminal-aware rendering.
#[derive(Default)]
pub struct ImagePreviewer {
    state: Mutex<ImagePreviewState>,
}

impl ImagePreviewer {
    pub fn render(&self, frame: &mut Frame, area: Rect, path: &Path, display_path: &str) -> bool {
        let backend = match backend::backend() {
            Some(backend) => backend,
            None => return false,
        };

        if !Self::supports(path) {
            return false;
        }

        frame.render_widget(Clear, area);
        if area.width == 0 || area.height == 0 {
            return true;
        }

        let key = ImageKey::new(path);
        let warning = backend.warning().map(str::to_owned);

        let mut state = self.state.lock().expect("image preview state poisoned");
        state.poll_pending(backend);

        if let Some(cache) = state.cached_for_mut(&key) {
            render_cached(frame, area, cache, display_path, warning.as_deref());
            return true;
        }

        state.ensure_request(key.clone());
        state.poll_pending(backend);

        if let Some(cache) = state.cached_for_mut(&key) {
            render_cached(frame, area, cache, display_path, warning.as_deref());
            return true;
        }

        let mut message = format!("Loading image preview for {display_path}");
        if let Some(extra) = &warning {
            message.push('\n');
            message.push_str(extra);
        }
        render_message(frame, area, &message);
        true
    }

    fn supports(path: &Path) -> bool {
        if !has_supported_extension(path) {
            return false;
        }

        match fs::metadata(path) {
            Ok(metadata) => metadata.is_file(),
            Err(_) => true,
        }
    }
}

/// Tracks cached render output and inflight decode jobs.
#[derive(Default)]
struct ImagePreviewState {
    cached: Option<CachedImage>,
    pending: Option<PendingImage>,
}

impl ImagePreviewState {
    fn poll_pending(&mut self, backend: &backend::GraphicsBackend) {
        let Some(pending) = self.pending.as_mut() else {
            return;
        };

        match pending.receiver.try_recv() {
            Ok(result) => {
                let key = pending.key.clone();
                self.pending = None;
                self.cached = Some(match result {
                    Ok(image) => CachedImage::Ready {
                        key,
                        protocol: backend.picker().new_resize_protocol(image),
                    },
                    Err(error) => CachedImage::Failed { key, error },
                });
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.pending = None;
            }
        }
    }

    fn cached_for_mut(&mut self, key: &ImageKey) -> Option<&mut CachedImage> {
        match self.cached.as_mut() {
            Some(cache) if cache.key() == key => Some(cache),
            _ => None,
        }
    }

    fn ensure_request(&mut self, key: ImageKey) {
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.key == key)
        {
            return;
        }

        let path = key.path.clone();
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let result = load_image(&path);
            let _ = sender.send(result);
        });

        self.pending = Some(PendingImage { key, receiver });
    }
}

struct PendingImage {
    key: ImageKey,
    receiver: Receiver<Result<DynamicImage, String>>,
}

enum CachedImage {
    Ready {
        key: ImageKey,
        protocol: StatefulProtocol,
    },
    Failed {
        key: ImageKey,
        error: String,
    },
}

impl CachedImage {
    fn key(&self) -> &ImageKey {
        match self {
            CachedImage::Ready { key, .. } | CachedImage::Failed { key, .. } => key,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ImageKey {
    path: PathBuf,
    modified: Option<SystemTime>,
}

impl ImageKey {
    fn new(path: &Path) -> Self {
        let modified = fs::metadata(path).and_then(|meta| meta.modified()).ok();
        Self {
            path: path.to_path_buf(),
            modified,
        }
    }
}

fn render_cached(
    frame: &mut Frame,
    area: Rect,
    cache: &mut CachedImage,
    display_path: &str,
    warning: Option<&str>,
) {
    match cache {
        CachedImage::Ready { protocol, .. } => {
            let widget: StatefulImage<StatefulProtocol> = StatefulImage::default();
            frame.render_stateful_widget(widget, area, protocol);
            if let Some(result) = protocol.last_encoding_result() {
                if let Err(error) = result {
                    let message = format!("Unable to render image {display_path}: {}", error);
                    render_message(frame, area, &message);
                }
            }
        }
        CachedImage::Failed { error, .. } => {
            let mut message = format!("Unable to preview {display_path}: {error}");
            if let Some(extra) = warning {
                message.push('\n');
                message.push_str(extra);
            }
            render_message(frame, area, &message);
        }
    }
}

fn render_message(frame: &mut Frame, area: Rect, message: &str) {
    frame.render_widget(Clear, area);
    let paragraph = Paragraph::new(message).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn has_supported_extension(path: &Path) -> bool {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => SUPPORTED_IMAGE_EXTENSIONS
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(ext)),
        None => false,
    }
}

fn load_image(path: &Path) -> Result<DynamicImage, String> {
    let reader = ImageReader::open(path).map_err(|err| err.to_string())?;
    reader.decode().map_err(|err| err.to_string())
}
