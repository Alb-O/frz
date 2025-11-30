//! A simple wrapper to run a command under `kitty --dump-commands=yes` and filter its output to only visible text.

#![allow(unused_crate_dependencies)]

use std::env;
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Command, Stdio};

fn main() -> io::Result<()> {
	let args: Vec<String> = env::args().skip(1).collect();

	if args.is_empty() {
		eprintln!("Usage: kitty-runner <command> [args...]");
		eprintln!("Example: kitty-runner cargo test");
		std::process::exit(1);
	}

	// Spawn the command with kitty --dump-commands=yes wrapping
	let mut kitty_cmd = Command::new("kitty");
	kitty_cmd
		.arg("--dump-commands=yes")
		.args(&args)
		.stdout(Stdio::piped())
		.stderr(Stdio::piped());

	let mut child = kitty_cmd.spawn()?;

	// Process stdout
	let stdout_handle = if let Some(stdout) = child.stdout.take() {
		let reader = BufReader::new(stdout);
		Some(std::thread::spawn(move || {
			let mut output = String::new();
			for line in reader.lines().map_while(Result::ok) {
				if line.starts_with("draw ") {
					// Extract the text after "draw " and add it to output
					if let Some(text) = line.strip_prefix("draw ") {
						output.push_str(text);
					}
				} else if line == "screen_linefeed" {
					// Add a newline when we see a linefeed command
					output.push('\n');
				}
				// Ignore screen_carriage_return and other commands
			}
			output
		}))
	} else {
		None
	};

	// Process stderr (pass through as-is for error messages)
	let stderr_handle = if let Some(stderr) = child.stderr.take() {
		let reader = BufReader::new(stderr);
		Some(std::thread::spawn(move || {
			for line in reader.lines().map_while(Result::ok) {
				eprintln!("{}", line);
			}
		}))
	} else {
		None
	};

	// Wait for the process to complete
	let status = child.wait()?;

	// Wait for both threads to finish
	if let Some(handle) = stderr_handle {
		handle.join().ok();
	}

	// Get the filtered output and print to stdout
	if let Some(handle) = stdout_handle
		&& let Ok(output) = handle.join()
	{
		let stdout = io::stdout();
		let mut handle = stdout.lock();
		write!(handle, "{}", output)?;
		handle.flush()?;
	}

	// Exit with the same status as the child process
	std::process::exit(status.code().unwrap_or(1))
}
