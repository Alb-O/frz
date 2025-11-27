use std::path::PathBuf;

use clap::builder::BoolishValueParser;
use clap::{ArgAction, ColorChoice, Parser};

use super::options::{OutputFormat, UiPresetArg};
use super::styles::{cli_styles, long_version};

/// Command-line arguments accepted by the `frz` binary.
#[derive(Parser, Debug)]
#[command(
    name = "frz",
    version,
    long_version = long_version(),
    about = "Interactive fuzzy finder for tabular data",
    color = ColorChoice::Auto,
    styles = cli_styles()
)]
pub(crate) struct CliArgs {
	#[arg(
        short,
        long = "config",
        value_name = "FILE",
        env = "FRZ_CONFIG",
        action = ArgAction::Append,
        help = "Additional configuration file to merge"
    )]
	pub(crate) config: Vec<PathBuf>,
	#[arg(
		short = 'n',
		long = "no-config",
		help = "Skip loading default configuration files"
	)]
	pub(crate) no_config: bool,
	#[arg(
		short = 'r',
		long,
		value_name = "PATH",
		help = "Override the filesystem root to scan"
	)]
	pub(crate) root: Option<PathBuf>,
	#[arg(
		short = 'q',
		long,
		value_name = "QUERY",
		help = "Provide an initial search query"
	)]
	pub(crate) initial_query: Option<String>,
	#[arg(long, value_name = "THEME", help = "Select a theme by name")]
	pub(crate) theme: Option<String>,
	#[arg(
		short = 'u',
		long = "ui-preset",
		value_enum,
		help = "Choose a preset for UI labels"
	)]
	pub(crate) ui_preset: Option<UiPresetArg>,
	#[arg(
		long = "filter-label",
		value_name = "TEXT",
		help = "Override the filter input label"
	)]
	pub(crate) filter_label: Option<String>,
	#[arg(
		long = "detail-title",
		value_name = "TEXT",
		help = "Override the detail panel title"
	)]
	pub(crate) detail_title: Option<String>,
	#[arg(
		long = "files-mode-title",
		value_name = "TEXT",
		help = "Set the files pane title"
	)]
	pub(crate) files_mode_title: Option<String>,
	#[arg(
		long = "files-hint",
		value_name = "TEXT",
		help = "Set the hint for the files pane"
	)]
	pub(crate) files_hint: Option<String>,
	#[arg(
		long = "files-table-title",
		value_name = "TEXT",
		help = "Set the table title for files"
	)]
	pub(crate) files_table_title: Option<String>,
	#[arg(
		long = "files-count-label",
		value_name = "TEXT",
		help = "Set the count label for files"
	)]
	pub(crate) files_count_label: Option<String>,
	#[arg(
		long = "file-headers",
		value_delimiter = ',',
		value_name = "HEADER",
		help = "Comma-separated file table headers"
	)]
	pub(crate) file_headers: Option<Vec<String>>,
	#[arg(
        short = 'H',
        long = "hidden",
        value_parser = BoolishValueParser::new(),
        help = "Include hidden files"
    )]
	pub(crate) hidden: Option<bool>,
	#[arg(
        short = 's',
        long = "follow-symlinks",
        value_parser = BoolishValueParser::new(),
        help = "Follow symbolic links while scanning"
    )]
	pub(crate) follow_symlinks: Option<bool>,
	#[arg(
        long = "respect-ignore-files",
        value_parser = BoolishValueParser::new(),
        help = "Respect .ignore files"
    )]
	pub(crate) respect_ignore_files: Option<bool>,
	#[arg(
        long = "git-ignore",
        value_parser = BoolishValueParser::new(),
        help = "Respect .gitignore files"
    )]
	pub(crate) git_ignore: Option<bool>,
	#[arg(
        long = "git-global",
        value_parser = BoolishValueParser::new(),
        help = "Respect global gitignore settings"
    )]
	pub(crate) git_global: Option<bool>,
	#[arg(
        long = "git-exclude",
        value_parser = BoolishValueParser::new(),
        help = "Respect git exclude files"
    )]
	pub(crate) git_exclude: Option<bool>,
	#[arg(
		short = 'j',
		long,
		value_name = "NUM",
		help = "Limit the number of indexing threads"
	)]
	pub(crate) threads: Option<usize>,
	#[arg(
		short = 'd',
		long = "max-depth",
		value_name = "NUM",
		help = "Limit directory traversal depth"
	)]
	pub(crate) max_depth: Option<usize>,
	#[arg(
		long = "extensions",
		value_delimiter = ',',
		value_name = "EXT",
		help = "Restrict search to specific file extensions"
	)]
	pub(crate) extensions: Option<Vec<String>>,
	#[arg(
		long = "context-label",
		value_name = "TEXT",
		help = "Override the context label shown in the prompt"
	)]
	pub(crate) context_label: Option<String>,
	#[arg(
		long = "global-ignores",
		value_delimiter = ',',
		value_name = "NAME",
		help = "Comma-separated directory names to always ignore"
	)]
	pub(crate) global_ignores: Option<Vec<String>>,
	#[arg(
		short = 'p',
		long = "print-config",
		help = "Print the resolved configuration before running"
	)]
	pub(crate) print_config: bool,
	#[arg(
		short = 'l',
		long = "list-themes",
		help = "List supported themes and exit"
	)]
	pub(crate) list_themes: bool,
	#[arg(
        short = 'o',
        long = "output",
        value_enum,
        default_value_t = OutputFormat::Plain,
        help = "Choose how to print the result"
    )]
	pub(crate) output: OutputFormat,
}
