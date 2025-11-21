use std::path::PathBuf;

use clap::Parser;

use super::RawConfig;
use crate::cli::CliArgs;

#[test]
fn cli_overrides_take_precedence() {
	let mut cli = CliArgs::parse_from(["frz"]);
	cli.root = Some(PathBuf::from("/tmp"));
	cli.hidden = Some(false);
	cli.follow_symlinks = Some(true);
	cli.respect_ignore_files = Some(false);
	cli.git_ignore = Some(false);
	cli.git_global = Some(false);
	cli.git_exclude = Some(false);
	cli.threads = Some(4);
	cli.max_depth = Some(10);
	cli.extensions = Some(vec!["rs".into()]);
	cli.global_ignores = Some(vec!["target".into()]);
	cli.context_label = Some("ctx".into());
	cli.title = Some("title".into());
	cli.initial_query = Some("query".into());
	cli.theme = Some("dark".into());
	cli.filter_label = Some("filter".into());
	cli.detail_title = Some("detail".into());
	cli.files_mode_title = Some("file".into());
	cli.files_hint = Some("file hint".into());
	cli.files_table_title = Some("files table".into());
	cli.files_count_label = Some("files count".into());
	cli.file_headers = Some(vec!["b".into()]);

	let mut config = RawConfig::default();
	config.apply_cli_overrides(&cli);

	assert_eq!(config.filesystem.root, cli.root);
	assert_eq!(config.ui.input_title, cli.title);
	assert_eq!(config.ui.initial_query, cli.initial_query);
	assert_eq!(config.ui.theme, cli.theme);
	assert_eq!(config.ui.file_headers, cli.file_headers);
}
