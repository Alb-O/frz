//! Basic demonstration of the kitty harness functionality.

#![allow(unused_crate_dependencies)]

use std::path::PathBuf;
use std::time::Duration;

use kitty_test_harness::{KeyPress, kitty_send_keys, with_kitty_capture};
use termwiz::input::KeyCode;

#[test]
#[ignore = "example test"]
fn basic_echo_capture() {
	let working_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

	let output = with_kitty_capture(&working_dir, "bash", |kitty| {
		// Send echo command and capture output
		kitty.send_text("echo 'Hello from kitty harness'\n");
		std::thread::sleep(Duration::from_millis(100));
		kitty.screen_text()
	});

	assert!(
		output.contains("Hello from kitty harness"),
		"Expected echo output to appear in screen capture"
	);
}

#[test]
#[ignore = "example test"]
fn key_press_navigation() {
	let working_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

	with_kitty_capture(&working_dir, "bash", |kitty| {
		// Create a multi-line output
		kitty.send_text("printf 'Line 1\\nLine 2\\nLine 3\\n'\n");
		std::thread::sleep(Duration::from_millis(100));

		// Send arrow keys using the macro
		kitty_send_keys!(kitty, KeyCode::UpArrow, KeyCode::UpArrow);
		std::thread::sleep(Duration::from_millis(50));

		let after = kitty.screen_text();

		// The screen should contain our output
		assert!(after.contains("Line 1"));
		assert!(after.contains("Line 2"));
		assert!(after.contains("Line 3"));
	});
}

#[test]
#[ignore = "example test"]
fn ansi_stripping() {
	let working_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

	with_kitty_capture(&working_dir, "bash", |kitty| {
		// Send colored output
		kitty.send_text("printf '\\033[31mRed text\\033[0m\\n'\n");
		std::thread::sleep(Duration::from_millis(100));

		let (raw, clean) = kitty.screen_text_clean();

		// Raw output should contain ANSI escape sequences
		assert!(raw.contains("\x1b["));

		// Clean output should not contain escape sequences
		assert!(clean.contains("Red text"));
		assert!(!clean.contains("\x1b["));
	});
}

#[test]
#[ignore = "example test"]
fn key_press_with_modifiers() {
	let working_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

	with_kitty_capture(&working_dir, "bash", |kitty| {
		use termwiz::input::Modifiers;

		// Send text
		kitty.send_text("hello world\n");
		std::thread::sleep(Duration::from_millis(50));

		// Send Ctrl+C using KeyPress
		let ctrl_c = KeyPress {
			key: KeyCode::Char('c'),
			mods: Modifiers::CTRL,
		};
		kitty_send_keys!(kitty, ctrl_c);
		std::thread::sleep(Duration::from_millis(50));

		let output = kitty.screen_text();
		assert!(output.contains("hello world"));
	});
}
