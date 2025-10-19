use std::path::PathBuf;

use clap::Parser;

use super::RawConfig;
use crate::cli::CliArgs;

#[test]
fn cli_overrides_take_precedence() {
    let mut cli = CliArgs::parse_from(["frz", "--start-mode", "files"]);
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
    cli.attributes_mode_title = Some("attribute".into());
    cli.attributes_hint = Some("hint".into());
    cli.attributes_table_title = Some("table".into());
    cli.attributes_count_label = Some("count".into());
    cli.files_mode_title = Some("file".into());
    cli.files_hint = Some("file hint".into());
    cli.files_table_title = Some("files table".into());
    cli.files_count_label = Some("files count".into());
    cli.facet_headers = Some(vec!["a".into()]);
    cli.file_headers = Some(vec!["b".into()]);
    cli.git_modifications = Some(false);

    let mut config = RawConfig::default();
    config.apply_cli_overrides(&cli);

    assert_eq!(config.filesystem.root, cli.root);
    assert_eq!(config.ui.input_title, cli.title);
    assert_eq!(config.ui.initial_query, cli.initial_query);
    assert_eq!(config.ui.theme, cli.theme);
    assert_eq!(config.ui.start_mode, Some("files".into()));
    assert_eq!(config.ui.facet_headers, cli.facet_headers);
    assert_eq!(config.ui.file_headers, cli.file_headers);
    assert_eq!(config.ui.git_modifications, cli.git_modifications);
}
