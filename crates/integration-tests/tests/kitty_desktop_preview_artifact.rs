//! Captures visual escape sequence artifacts in frz against ~/desktop.

use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use kitty_test_harness::{kitty_send_keys, with_kitty_capture};
use termwiz::input::KeyCode;

const FAIL_ON_RAW: Option<&str> = None;
const FAIL_ON_CLEAN: Option<&str> = Some(r"t split:");

#[test]
#[ignore = "needs to be ran from albert's machine"]
fn t01_fail_on_string() {
	let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	let home = std::env::var("HOME").expect("HOME is set");
	let desktop = PathBuf::from(home).join("desktop");
	assert!(
		desktop.is_dir(),
		"expected ~/desktop to exist so the test matches the manual repro"
	);

	build_frz(&workspace_root);

	let shell_cmd = format!(
		"FRZ_PREVIEW_MAX_IMAGE_BYTES=500000 FRZ_PREVIEW_IMAGE_ENCODE_CELLS=20x20 FRZ_TUI_MAX_SIZE=100x32 target/release/frz -r {}",
		desktop.display(),
	);

	with_kitty_capture(&workspace_root, &shell_cmd, |kitty| {
		let before = kitty.screen_text();
		kitty_send_keys!(kitty, KeyCode::DownArrow, KeyCode::DownArrow,);
		std::thread::sleep(Duration::from_millis(800));
		let (after_raw, after_clean) = kitty.screen_text_clean();
		assert_ne!(
			before, after_raw,
			"expected Down keys to change the screen output"
		);

		println!("AFTER_RAW:\n{after_raw}\n---\nAFTER_CLEAN:\n{after_clean}");

		if let Some(needle) = FAIL_ON_RAW
			&& after_raw.contains(needle)
		{
			panic!("found forbidden pattern '{needle}' in raw output");
		}

		if let Some(needle) = FAIL_ON_CLEAN
			&& after_clean.contains(needle)
		{
			panic!("found forbidden pattern '{needle}' in cleaned output");
		}

		"ok".to_string()
	});
}

fn build_frz(workspace_root: &std::path::Path) {
	let status = Command::new("cargo")
		.current_dir(workspace_root)
		.args([
			"build",
			"-p",
			"frz-cli",
			"--features",
			"frz-tui/media-preview",
			"--release",
		])
		.status()
		.expect("cargo build should run");
	assert!(status.success(), "cargo build should succeed");
}
