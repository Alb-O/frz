#[cfg(feature = "fs")]
use std::fmt::Write;
#[cfg(feature = "fs")]
use std::path::PathBuf;

#[cfg(feature = "fs")]
use clap::{
    ArgAction, ColorChoice, Command, CommandFactory, FromArgMatches, Parser, ValueEnum,
    builder::{
        BoolishValueParser, Styles,
        styling::{AnsiColor, Effects},
    },
};
#[cfg(feature = "fs")]
use frz::app_dirs;

#[cfg(feature = "fs")]
use super::annotations::dim_cli_annotations;

#[cfg(feature = "fs")]
/// Produce the full version banner including config and data directories.
fn long_version() -> &'static str {
    let config_dir = match app_dirs::get_config_dir() {
        Ok(path) => path.display().to_string(),
        Err(err) => format!("unavailable ({err})"),
    };
    let data_dir = match app_dirs::get_data_dir() {
        Ok(path) => path.display().to_string(),
        Err(err) => format!("unavailable ({err})"),
    };

    let mut details = format!("frz {}", env!("CARGO_PKG_VERSION"));
    let _ = writeln!(details);
    let _ = writeln!(details, "config directory: {config_dir}");
    let _ = writeln!(details, "data directory: {data_dir}");

    Box::leak(details.into_boxed_str())
}

#[cfg(feature = "fs")]
/// Create the clap styles used for custom colour output.
fn cli_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
        .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
        .literal(AnsiColor::Cyan.on_default())
        .placeholder(AnsiColor::Yellow.on_default())
}

#[cfg(feature = "fs")]
/// Parse command line arguments into the strongly typed [`CliArgs`] structure.
pub(crate) fn parse_cli() -> CliArgs {
    let mut matches = tinted_cli_command().get_matches();
    CliArgs::from_arg_matches_mut(&mut matches).unwrap_or_else(|err| err.exit())
}

#[cfg(feature = "fs")]
/// Apply styling customisation to the generated clap command.
fn tinted_cli_command() -> Command {
    CliArgs::command().mut_args(dim_cli_annotations)
}

#[cfg(feature = "fs")]
#[derive(Parser, Debug)]
#[command(
    name = "frz",
    version,
    long_version = long_version(),
    about = "Interactive fuzzy finder for tabular data",
    color = ColorChoice::Auto,
    styles = cli_styles()
)]
/// Command-line arguments accepted by the `frz` binary.
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
        long = "facets-mode-title",
        value_name = "TEXT",
        help = "Set the facets pane title (default: preset value)"
    )]
    pub(crate) facets_mode_title: Option<String>,
    #[arg(
        long = "facets-hint",
        value_name = "TEXT",
        help = "Set the hint for the facets pane (default: preset value)"
    )]
    pub(crate) facets_hint: Option<String>,
    #[arg(
        long = "facets-table-title",
        value_name = "TEXT",
        help = "Set the table title for facets (default: preset value)"
    )]
    pub(crate) facets_table_title: Option<String>,
    #[arg(
        long = "facets-count-label",
        value_name = "TEXT",
        help = "Set the count label for facets (default: preset value)"
    )]
    pub(crate) facets_count_label: Option<String>,
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
        long = "facet-headers",
        value_delimiter = ',',
        value_name = "HEADER",
        help = "Comma-separated facet table headers (default: preset value)"
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
    #[arg(short = 'o', long = "output", value_enum, default_value_t = OutputFormat::Plain, help = "Choose how to print the result")]
    pub(crate) output: OutputFormat,
}

#[cfg(feature = "fs")]
#[derive(Copy, Clone, Debug, ValueEnum)]
/// Search modes accepted via the command line.
pub(crate) enum ModeArg {
    Facets,
    Files,
}

#[cfg(feature = "fs")]
impl ModeArg {
    /// Return the string representation consumed by configuration loading.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ModeArg::Facets => "facets",
            ModeArg::Files => "files",
        }
    }
}

#[cfg(feature = "fs")]
#[derive(Copy, Clone, Debug, ValueEnum)]
/// Predefined UI presets selectable from the CLI.
pub(crate) enum UiPresetArg {
    Default,
    #[clap(name = "tags-and-files")]
    TagsAndFiles,
}

#[cfg(feature = "fs")]
impl UiPresetArg {
    /// Return the preset identifier consumed by configuration loading.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            UiPresetArg::Default => "default",
            UiPresetArg::TagsAndFiles => "tags-and-files",
        }
    }
}

#[cfg(feature = "fs")]
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
/// Output formats supported by the CLI utility.
pub(crate) enum OutputFormat {
    Plain,
    Json,
}

#[cfg(all(test, feature = "fs"))]
mod tests {
    use super::*;

    #[test]
    fn command_supports_custom_styles() {
        let command = tinted_cli_command();
        assert!(command.get_about().is_some());
    }

    #[test]
    fn parse_cli_accepts_default_arguments() {
        let command = CliArgs::command();
        let mut matches = command.get_matches_from(vec!["frz"]);
        let parsed = CliArgs::from_arg_matches_mut(&mut matches).expect("parses");
        assert_eq!(parsed.output, OutputFormat::Plain);
    }
}
