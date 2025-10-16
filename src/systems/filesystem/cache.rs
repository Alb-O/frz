use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::app_dirs;
use frz_plugin_api::{AttributeRow, FileRow, SearchData};

use super::FilesystemOptions;

pub(super) const CACHE_TTL: Duration = Duration::from_secs(60);
const CACHE_VERSION: u32 = 1;
const CACHE_NAMESPACE: &str = "filesystem";
const CACHE_PREVIEW_LIMIT: usize = 512;
const CACHE_PREVIEW_EXTENSION: &str = "preview.json";

#[derive(Clone)]
pub(super) struct CacheHandle {
    path: PathBuf,
    fingerprint: u64,
}

pub(super) struct CachedEntry {
    pub data: SearchData,
    pub indexed_at: SystemTime,
    pub complete: bool,
}

impl CachedEntry {
    pub fn reindex_delay(&self) -> Duration {
        match SystemTime::now().duration_since(self.indexed_at) {
            Ok(age) => CACHE_TTL.saturating_sub(age),
            Err(_) => Duration::ZERO,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.complete
    }
}

impl CacheHandle {
    pub fn resolve(root: &Path, options: &FilesystemOptions) -> Option<Self> {
        let base = app_dirs::get_cache_dir().ok()?;
        let fingerprint = fingerprint_for(root, options);
        let file_name = format!("{fingerprint:016x}.json");
        let path = base.join(CACHE_NAMESPACE).join(file_name);
        Some(Self { path, fingerprint })
    }

    pub fn load(&self) -> Option<CachedEntry> {
        load_payload(&self.path, self.fingerprint)
    }

    pub fn writer(&self, context_label: Option<String>) -> Option<CacheWriter> {
        Some(CacheWriter::new(
            self.path.clone(),
            self.fingerprint,
            context_label,
        ))
    }

    pub fn load_preview(&self) -> Option<CachedEntry> {
        let preview_path = self.preview_path();
        load_payload(&preview_path, self.fingerprint)
    }

    fn preview_path(&self) -> PathBuf {
        let mut preview_path = self.path.clone();
        preview_path.set_extension(CACHE_PREVIEW_EXTENSION);
        preview_path
    }
}

pub(super) struct CacheWriter {
    path: PathBuf,
    fingerprint: u64,
    context_label: Option<String>,
    files: Vec<CacheFileEntry>,
    attributes: BTreeMap<String, usize>,
    preview_path: PathBuf,
}

impl CacheWriter {
    fn new(path: PathBuf, fingerprint: u64, context_label: Option<String>) -> Self {
        let mut preview_path = path.clone();
        preview_path.set_extension(CACHE_PREVIEW_EXTENSION);
        Self {
            path,
            fingerprint,
            context_label,
            files: Vec::new(),
            attributes: BTreeMap::new(),
            preview_path,
        }
    }

    pub fn record(&mut self, file: &FileRow) {
        self.files.push(CacheFileEntry {
            path: file.path.clone(),
            tags: file.tags.clone(),
        });

        for tag in &file.tags {
            let count = self.attributes.entry(tag.clone()).or_insert(0);
            *count += 1;
        }
    }

    pub fn finish(self) -> Result<()> {
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir)
                .with_context(|| format!("failed to create cache directory: {}", dir.display()))?;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let attributes = self
            .attributes
            .into_iter()
            .map(|(name, count)| CacheAttributeEntry { name, count })
            .collect::<Vec<_>>();

        let preview_files: Vec<CacheFileEntry> = self
            .files
            .iter()
            .take(CACHE_PREVIEW_LIMIT)
            .cloned()
            .collect();
        let mut preview_attributes = BTreeMap::new();
        for file in &preview_files {
            for tag in &file.tags {
                *preview_attributes.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        let preview_attributes = preview_attributes
            .into_iter()
            .map(|(name, count)| CacheAttributeEntry { name, count })
            .collect::<Vec<_>>();
        let preview_complete = preview_files.len() == self.files.len();

        let payload = CachePayload {
            version: CACHE_VERSION,
            fingerprint: self.fingerprint,
            indexed_at: timestamp,
            context_label: self.context_label.clone(),
            complete: true,
            files: self.files,
            attributes,
        };

        let preview_payload = CachePayload {
            version: CACHE_VERSION,
            fingerprint: self.fingerprint,
            indexed_at: timestamp,
            context_label: self.context_label,
            complete: preview_complete,
            files: preview_files,
            attributes: preview_attributes,
        };

        write_payload(&self.path, &payload)?;
        write_payload(&self.preview_path, &preview_payload)
    }
}

#[derive(Serialize, Deserialize)]
struct CachePayload {
    version: u32,
    fingerprint: u64,
    indexed_at: u64,
    context_label: Option<String>,
    #[serde(default)]
    complete: bool,
    files: Vec<CacheFileEntry>,
    attributes: Vec<CacheAttributeEntry>,
}

#[derive(Serialize, Deserialize, Clone)]
struct CacheFileEntry {
    path: String,
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct CacheAttributeEntry {
    name: String,
    count: usize,
}

fn write_payload(path: &Path, payload: &CachePayload) -> Result<()> {
    let data = serde_json::to_vec(payload).context("failed to serialize cache payload")?;
    let tmp_path = path.with_extension("tmp");
    {
        let mut file = fs::File::create(&tmp_path)
            .with_context(|| format!("failed to create cache file: {}", tmp_path.display()))?;
        file.write_all(&data)
            .with_context(|| format!("failed to write cache file: {}", tmp_path.display()))?;
        file.sync_all().ok();
    }

    let _ = fs::remove_file(path);
    fs::rename(&tmp_path, path).with_context(|| {
        format!(
            "failed to move cache file from {} to {}",
            tmp_path.display(),
            path.display()
        )
    })?;

    Ok(())
}

fn load_payload(path: &Path, fingerprint: u64) -> Option<CachedEntry> {
    let bytes = fs::read(path).ok()?;
    let payload: CachePayload = serde_json::from_slice(&bytes).ok()?;
    if payload.version != CACHE_VERSION || payload.fingerprint != fingerprint {
        return None;
    }

    let indexed_at = UNIX_EPOCH + Duration::from_secs(payload.indexed_at);
    let mut data = SearchData::new();
    data.context_label = payload.context_label;
    data.files = payload
        .files
        .into_iter()
        .map(|entry| FileRow::filesystem(entry.path, entry.tags))
        .collect();
    data.attributes = payload
        .attributes
        .into_iter()
        .map(|entry| AttributeRow::new(entry.name, entry.count))
        .collect();

    Some(CachedEntry {
        data,
        indexed_at,
        complete: payload.complete,
    })
}

fn fingerprint_for(root: &Path, options: &FilesystemOptions) -> u64 {
    let mut hasher = DefaultHasher::new();
    root.to_string_lossy().hash(&mut hasher);
    options.include_hidden.hash(&mut hasher);
    options.follow_symlinks.hash(&mut hasher);
    options.respect_ignore_files.hash(&mut hasher);
    options.git_ignore.hash(&mut hasher);
    options.git_global.hash(&mut hasher);
    options.git_exclude.hash(&mut hasher);
    options.threads.hash(&mut hasher);
    options.max_depth.hash(&mut hasher);

    match options.allowed_extensions.as_ref() {
        Some(exts) => {
            1u8.hash(&mut hasher);
            let mut sorted = exts.clone();
            sorted.sort();
            sorted.hash(&mut hasher);
        }
        None => {
            0u8.hash(&mut hasher);
        }
    }

    let mut ignores = options.global_ignores.clone();
    ignores.sort();
    ignores.hash(&mut hasher);

    hasher.finish()
}
