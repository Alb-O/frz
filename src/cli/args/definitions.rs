use std::path::PathBuf;

use clap::builder::BoolishValueParser;
use clap::{ArgAction, ColorChoice, Parser};

use super::options::{ModeArg, OutputFormat, UiPresetArg};
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
        help = "Additional configuration file to merge (default: none)"
    )]
    pub(crate) config: Vec<PathBuf>,
    #[arg(
        short = 'n',
        long = "no-config",
        help = "Skip loading default configuration files (default: disabled)"
    )]
    pub(crate) no_config: bool,
    #[arg(
        short = 'r',
        long,
        value_name = "PATH",
        help = "Override the filesystem root to scan (default: current directory)"
    )]
    pub(crate) root: Option<PathBuf>,
    #[arg(
        short = 't',
        long,
        value_name = "TITLE",
        help = "Set the input prompt title (default: derived from root or context label)"
    )]
    pub(crate) title: Option<String>,
    #[arg(
        short = 'q',
        long,
        value_name = "QUERY",
        help = "Provide an initial search query (default: empty)"
    )]
    pub(crate) initial_query: Option<String>,
    #[arg(
        long,
        value_name = "THEME",
        help = "Select a theme by name (default: library theme)"
    )]
    pub(crate) theme: Option<String>,
    #[arg(
        short = 'm',
        long = "start-mode",
        value_enum,
        help = "Choose the initial search mode (default: match the library heuristic)"
    )]
    pub(crate) start_mode: Option<ModeArg>,
    #[arg(
        short = 'u',
        long = "ui-preset",
        value_enum,
        help = "Choose a preset for UI labels (default: default)"
    )]
    pub(crate) ui_preset: Option<UiPresetArg>,
    #[arg(
        long = "filter-label",
        value_name = "TEXT",
        help = "Override the filter input label (default: preset value)"
    )]
    pub(crate) filter_label: Option<String>,
    #[arg(
        long = "detail-title",
        value_name = "TEXT",
        help = "Override the detail panel title (default: preset value)"
    )]
    pub(crate) detail_title: Option<String>,
    #[arg(
        long = "attributes-mode-title",
        value_name = "TEXT",
        help = "Set the attributes pane title (default: preset value)"
    )]
    pub(crate) attributes_mode_title: Option<String>,
    #[arg(
        long = "attributes-hint",
        value_name = "TEXT",
        help = "Set the hint for the attributes pane (default: preset value)"
    )]
    pub(crate) attributes_hint: Option<String>,
    #[arg(
        long = "attributes-table-title",
        value_name = "TEXT",
        help = "Set the table title for attributes (default: preset value)"
    )]
    pub(crate) attributes_table_title: Option<String>,
    #[arg(
        long = "attributes-count-label",
        value_name = "TEXT",
        help = "Set the count label for attributes (default: preset value)"
    )]
    pub(crate) attributes_count_label: Option<String>,
    #[arg(
        long = "files-mode-title",
        value_name = "TEXT",
        help = "Set the files pane title (default: preset value)"
    )]
    pub(crate) files_mode_title: Option<String>,
    #[arg(
        long = "files-hint",
        value_name = "TEXT",
        help = "Set the hint for the files pane (default: preset value)"
    )]
    pub(crate) files_hint: Option<String>,
    #[arg(
        long = "files-table-title",
        value_name = "TEXT",
        help = "Set the table title for files (default: preset value)"
    )]
    pub(crate) files_table_title: Option<String>,
    #[arg(
        long = "files-count-label",
        value_name = "TEXT",
        help = "Set the count label for files (default: preset value)"
    )]
    pub(crate) files_count_label: Option<String>,
    #[arg(
        long = "attribute-headers",
        value_delimiter = ',',
        value_name = "HEADER",
        help = "Comma-separated attribute table headers (default: preset value)"
    )]
    pub(crate) facet_headers: Option<Vec<String>>,
    #[arg(
        long = "file-headers",
        value_delimiter = ',',
        value_name = "HEADER",
        help = "Comma-separated file table headers (default: preset value)"
    )]
    pub(crate) file_headers: Option<Vec<String>>,
    #[arg(
        short = 'H',
        long = "hidden",
        value_parser = BoolishValueParser::new(),
        help = "Include hidden files (default: enabled)"
    )]
    pub(crate) hidden: Option<bool>,
    #[arg(
        short = 's',
        long = "follow-symlinks",
        value_parser = BoolishValueParser::new(),
        help = "Follow symbolic links while scanning (default: disabled)"
    )]
    pub(crate) follow_symlinks: Option<bool>,
    #[arg(
        long = "respect-ignore-files",
        value_parser = BoolishValueParser::new(),
        help = "Respect .ignore files (default: enabled)"
    )]
    pub(crate) respect_ignore_files: Option<bool>,
    #[arg(
        long = "git-ignore",
        value_parser = BoolishValueParser::new(),
        help = "Respect .gitignore files (default: enabled)"
    )]
    pub(crate) git_ignore: Option<bool>,
    #[arg(
        long = "git-global",
        value_parser = BoolishValueParser::new(),
        help = "Respect global gitignore settings (default: enabled)"
    )]
    pub(crate) git_global: Option<bool>,
    #[arg(
        long = "git-exclude",
        value_parser = BoolishValueParser::new(),
        help = "Respect git exclude files (default: enabled)"
    )]
    pub(crate) git_exclude: Option<bool>,
    #[arg(
        short = 'j',
        long,
        value_name = "NUM",
        help = "Limit the number of indexing threads (default: automatic)"
    )]
    pub(crate) threads: Option<usize>,
    #[arg(
        short = 'd',
        long = "max-depth",
        value_name = "NUM",
        help = "Limit directory traversal depth (default: unlimited)"
    )]
    pub(crate) max_depth: Option<usize>,
    #[arg(
        long = "extensions",
        value_delimiter = ',',
        value_name = "EXT",
        help = "Restrict search to specific file extensions (default: all)"
    )]
    pub(crate) extensions: Option<Vec<String>>,
    #[arg(
        long = "context-label",
        value_name = "TEXT",
        help = "Override the context label shown in the prompt (default: derived from filesystem root)"
    )]
    pub(crate) context_label: Option<String>,
    #[arg(
        long = "global-ignores",
        value_delimiter = ',',
        value_name = "NAME",
        help = "Comma-separated directory names to always ignore (default: .git,node_modules,target,.venv)"
    )]
    pub(crate) global_ignores: Option<Vec<String>>,
    #[arg(
        short = 'p',
        long = "print-config",
        help = "Print the resolved configuration before running (default: disabled)"
    )]
    pub(crate) print_config: bool,
    #[arg(
        short = 'l',
        long = "list-themes",
        help = "List supported themes and exit (default: disabled)"
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
