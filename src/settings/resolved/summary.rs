use super::ResolvedConfig;

pub(super) fn print_summary(config: &ResolvedConfig) {
	println!("Effective configuration:");
	println!("  Root: {}", config.root.display());
	println!(
		"  Include hidden: {}",
		bool_to_word(config.filesystem.include_hidden)
	);
	println!(
		"  Follow symlinks: {}",
		bool_to_word(config.filesystem.follow_symlinks)
	);
	println!(
		"  Respect ignore files: {}",
		bool_to_word(config.filesystem.respect_ignore_files)
	);
	println!(
		"  Git ignore: {}",
		bool_to_word(config.filesystem.git_ignore)
	);
	println!(
		"  Git global: {}",
		bool_to_word(config.filesystem.git_global)
	);
	println!(
		"  Git exclude: {}",
		bool_to_word(config.filesystem.git_exclude)
	);
	match config.filesystem.max_depth {
		Some(depth) => println!("  Max depth: {depth}"),
		None => println!("  Max depth: unlimited"),
	}
	match &config.filesystem.allowed_extensions {
		Some(exts) if !exts.is_empty() => {
			println!("  Allowed extensions: {}", exts.join(", "));
		}
		_ => println!("  Allowed extensions: (all)"),
	}
	if let Some(threads) = config.filesystem.threads {
		println!("  Threads: {threads}");
	}
	if let Some(label) = &config.filesystem.context_label {
		println!("  Context label: {label}");
	}
	if !config.filesystem.global_ignores.is_empty() {
		println!(
			"  Global ignores: {}",
			config.filesystem.global_ignores.join(", ")
		);
	}
	println!(
		"  UI theme: {}",
		config
			.theme
			.as_deref()
			.unwrap_or("(use the library default)")
	);
	println!(
		"  Start mode: {}",
		config
			.start_mode
			.map(|mode| mode.id().to_string())
			.unwrap_or_else(|| "(auto)".to_string())
	);
	if let Some(title) = &config.input_title {
		println!("  Prompt title: {title}");
	}
	if !config.initial_query.is_empty() {
		println!("  Initial query: {}", config.initial_query);
	}
	if let Some(headers) = &config.facet_headers {
		println!("  attribute headers: {}", headers.join(", "));
	}
	if let Some(headers) = &config.file_headers {
		println!("  File headers: {}", headers.join(", "));
	}
}

fn bool_to_word(value: bool) -> &'static str {
	if value { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
	use std::path::PathBuf;

	use frz::{FilesystemOptions, UiConfig};

	use super::*;

	#[test]
	fn bool_to_word_matches_expectations() {
		assert_eq!(super::bool_to_word(true), "yes");
		assert_eq!(super::bool_to_word(false), "no");
	}

	#[test]
	fn summary_prints_without_panic() {
		let config = ResolvedConfig {
			root: PathBuf::from("/tmp"),
			filesystem: FilesystemOptions::default(),
			input_title: Some("Title".into()),
			initial_query: "foo".into(),
			theme: Some("dark".into()),
			start_mode: None,
			ui: UiConfig::default(),
			facet_headers: Some(vec!["col1".into()]),
			file_headers: Some(vec!["col2".into()]),
		};

		print_summary(&config);
	}
}
