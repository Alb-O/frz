use std::io::Write;
use std::process::{Command, Stdio};

/// Copy text to clipboard using available methods.
/// Tries OSC52 first (works in tmux/ssh), then falls back to native tools.
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
	if try_osc52_copy(text) {
		return Ok(());
	}
	try_native_clipboard(text)
}

fn try_osc52_copy(text: &str) -> bool {
	use base64::Engine;
	let encoded = base64::engine::general_purpose::STANDARD.encode(text);

	let osc52 = if std::env::var("TMUX").is_ok() {
		format!("\x1bPtmux;\x1b\x1b]52;c;{}\x07\x1b\\", encoded)
	} else {
		format!("\x1b]52;c;{}\x07", encoded)
	};

	let mut stdout = std::io::stdout().lock();
	stdout.write_all(osc52.as_bytes()).is_ok() && stdout.flush().is_ok()
}

fn try_native_clipboard(text: &str) -> Result<(), String> {
	let try_command = |cmd: &str, args: &[&str]| -> bool {
		Command::new(cmd)
			.args(args)
			.stdin(Stdio::piped())
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.spawn()
			.ok()
			.and_then(|mut child| {
				let success = child
					.stdin
					.take()
					.map(|mut stdin| stdin.write_all(text.as_bytes()).is_ok())
					.unwrap_or(false);
				if success {
					child.wait().ok().map(|_| ())
				} else {
					None
				}
			})
			.is_some()
	};

	if std::env::var("WAYLAND_DISPLAY").is_ok() && try_command("wl-copy", &[]) {
		return Ok(());
	}

	if try_command("xclip", &["-selection", "clipboard"]) {
		return Ok(());
	}

	if try_command("xsel", &["--clipboard", "--input"]) {
		return Ok(());
	}

	if try_command("pbcopy", &[]) {
		return Ok(());
	}

	Err("No clipboard tool available".to_string())
}
