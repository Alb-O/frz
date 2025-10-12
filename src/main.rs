use std::path::PathBuf;

#[cfg(feature = "fs")]
mod settings;

#[cfg(feature = "fs")]
use frz::app_dirs;

#[cfg(feature = "fs")]
fn long_version() -> &'static str {
    use std::fmt::Write;

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
use anyhow::Result;
#[cfg(feature = "fs")]
use clap::{
    ArgAction, ColorChoice, Command, CommandFactory, FromArgMatches, Parser, ValueEnum,
    builder::{
        BoolishValueParser, StyledStr,
        styling::{AnsiColor, Color, Effects, Style, Styles},
    },
};
#[cfg(feature = "fs")]
use serde_json::json;

#[cfg(feature = "fs")]
use frz::types::{SearchOutcome, SearchSelection};
#[cfg(feature = "fs")]
use frz::{SearchMode, Searcher};

#[cfg(feature = "fs")]
use settings::ResolvedConfig;

#[cfg(feature = "fs")]
fn cli_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
        .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
        .literal(AnsiColor::Cyan.on_default())
        .placeholder(AnsiColor::Yellow.on_default())
}

#[cfg(feature = "fs")]
fn parse_cli() -> CliArgs {
    let mut matches = tinted_cli_command().get_matches();
    CliArgs::from_arg_matches_mut(&mut matches).unwrap_or_else(|err| err.exit())
}

#[cfg(feature = "fs")]
fn tinted_cli_command() -> Command {
    CliArgs::command().mut_args(dim_cli_annotations)
}

#[cfg(feature = "fs")]
fn dim_cli_annotations(mut arg: clap::Arg) -> clap::Arg {
    let help_text = arg
        .get_help()
        .cloned()
        .map(|help| help.to_string())
        .unwrap_or_default();
    let mut styled_help = style_base_help(&help_text);
    let mut has_help = !help_text.is_empty();
    let mentions_default = help_text.contains("(default:");

    if let Some(annotation) = render_possible_values_annotation(&arg) {
        arg = arg.hide_possible_values(true);
        if has_help {
            styled_help.push_str(" ");
        }
        append_muted_annotation(&mut styled_help, &annotation);
        has_help = true;
    }

    if !mentions_default && let Some(annotation) = render_default_value_annotation(&arg) {
        arg = arg.hide_default_value(true);
        if has_help {
            styled_help.push_str(" ");
        }
        append_muted_annotation(&mut styled_help, &annotation);
        has_help = true;
    }

    // Append environment variable annotation (e.g. "[env: FRZ_CONFIG=]") when present
    if let Some(annotation) = render_env_annotation(&arg) {
        // Hide clap's built-in env hint so we can append our dimmed version
        arg = arg.hide_env(true);
        if has_help {
            styled_help.push_str(" ");
        }
        append_muted_annotation(&mut styled_help, &annotation);
        has_help = true;
    }

    if has_help {
        arg = arg.help(styled_help);
    }

    arg
}

#[cfg(feature = "fs")]
fn highlight_help_annotations(text: &str) -> Option<StyledStr> {
    const ANNOTATIONS: &[(&str, char)] = &[("(default: ", ')')];

    let mut styled = StyledStr::new();
    let mut last = 0;
    let mut changed = false;
    let mut cursor = 0;

    while cursor < text.len() {
        let mut best_match: Option<(usize, usize)> = None;

        for &(pattern, terminator) in ANNOTATIONS {
            if let Some(rel_start) = text[cursor..].find(pattern) {
                let start = cursor + rel_start;
                if let Some(rel_end) = text[start..].find(terminator) {
                    let end = start + rel_end + 1;
                    if best_match.is_none_or(|(current_start, _)| start < current_start) {
                        best_match = Some((start, end));
                    }
                }
            }
        }

        let (start, end) = match best_match {
            Some(bounds) => bounds,
            None => break,
        };

        styled.push_str(&text[last..start]);

        let style = muted_default_style();
        std::fmt::write(
            &mut styled,
            format_args!("{style}{}{style:#}", &text[start..end]),
        )
        .ok()?;

        last = end;
        cursor = end;
        changed = true;
    }

    if !changed {
        return None;
    }

    styled.push_str(&text[last..]);
    Some(styled)
}

#[cfg(feature = "fs")]
fn muted_default_style() -> Style {
    Style::new()
        .fg_color(Some(Color::Ansi(AnsiColor::BrightBlack)))
        .dimmed()
}

#[cfg(feature = "fs")]
fn style_base_help(text: &str) -> StyledStr {
    if text.is_empty() {
        return StyledStr::new();
    }

    highlight_help_annotations(text).unwrap_or_else(|| {
        let mut styled = StyledStr::new();
        styled.push_str(text);
        styled
    })
}

#[cfg(feature = "fs")]
fn append_muted_annotation(target: &mut StyledStr, annotation: &str) {
    let style = muted_default_style();
    let _ = std::fmt::write(target, format_args!("{style}{annotation}{style:#}"));
}

#[cfg(feature = "fs")]
fn render_possible_values_annotation(arg: &clap::Arg) -> Option<String> {
    if !arg.get_action().takes_values() {
        return None;
    }

    let values = arg.get_possible_values();
    if values.is_empty() {
        return None;
    }

    let mut visible = Vec::new();
    for value in values {
        if value.is_hide_set() {
            continue;
        }

        let name = value.get_name();
        let formatted = if name.chars().any(char::is_whitespace) {
            format!("{name:?}")
        } else {
            name.to_string()
        };
        visible.push(formatted);
    }

    if visible.is_empty() {
        return None;
    }

    Some(format!("[possible values: {}]", visible.join(", ")))
}

#[cfg(feature = "fs")]
fn render_default_value_annotation(arg: &clap::Arg) -> Option<String> {
    let defaults = arg.get_default_values();
    if defaults.is_empty() {
        return None;
    }

    let mut rendered = Vec::new();
    for value in defaults {
        let text = value.to_string_lossy();
        if text.trim().is_empty() {
            continue;
        }

        let formatted = if text.chars().any(char::is_whitespace) {
            format!("{text:?}")
        } else {
            text.to_string()
        };
        rendered.push(formatted);
    }

    if rendered.is_empty() {
        return None;
    }

    Some(format!("(default: {})", rendered.join(", ")))
}

#[cfg(feature = "fs")]
fn render_env_annotation(arg: &clap::Arg) -> Option<String> {
    // clap exposes environment variable info via get_env. If present, show as
    // `[env: NAME=]` (no value printed) to hint that the arg can be set via env.
    if let Some(env) = arg.get_env() {
        // get_env returns an OsStr; convert to a string lossily for display
        let name = env.to_string_lossy();
        if name.trim().is_empty() {
            return None;
        }

        Some(format!("[env: {}=]", name))
    } else {
        None
    }
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
struct CliArgs {
    #[arg(
        short,
        long = "config",
        value_name = "FILE",
        env = "FRZ_CONFIG",
        action = ArgAction::Append,
        help = "Additional configuration file to merge (default: none)"
    )]
    config: Vec<PathBuf>,
    #[arg(
        short = 'n',
        long = "no-config",
        help = "Skip loading default configuration files (default: disabled)"
    )]
    no_config: bool,
    #[arg(
        short = 'r',
        long,
        value_name = "PATH",
        help = "Override the filesystem root to scan (default: current directory)"
    )]
    root: Option<PathBuf>,
    #[arg(
        short = 't',
        long,
        value_name = "TITLE",
        help = "Set the input prompt title (default: derived from root or context label)"
    )]
    title: Option<String>,
    #[arg(
        short = 'q',
        long,
        value_name = "QUERY",
        help = "Provide an initial search query (default: empty)"
    )]
    initial_query: Option<String>,
    #[arg(
        long,
        value_name = "THEME",
        help = "Select a theme by name (default: library theme)"
    )]
    theme: Option<String>,
    #[arg(
        short = 'm',
        long = "start-mode",
        value_enum,
        help = "Choose the initial search mode (default: match the library heuristic)"
    )]
    start_mode: Option<ModeArg>,
    #[arg(
        short = 'u',
        long = "ui-preset",
        value_enum,
        help = "Choose a preset for UI labels (default: default)"
    )]
    ui_preset: Option<UiPresetArg>,
    #[arg(
        long = "filter-label",
        value_name = "TEXT",
        help = "Override the filter input label (default: preset value)"
    )]
    filter_label: Option<String>,
    #[arg(
        long = "detail-title",
        value_name = "TEXT",
        help = "Override the detail panel title (default: preset value)"
    )]
    detail_title: Option<String>,
    #[arg(
        long = "facets-mode-title",
        value_name = "TEXT",
        help = "Set the facets pane title (default: preset value)"
    )]
    facets_mode_title: Option<String>,
    #[arg(
        long = "facets-hint",
        value_name = "TEXT",
        help = "Set the hint for the facets pane (default: preset value)"
    )]
    facets_hint: Option<String>,
    #[arg(
        long = "facets-table-title",
        value_name = "TEXT",
        help = "Set the table title for facets (default: preset value)"
    )]
    facets_table_title: Option<String>,
    #[arg(
        long = "facets-count-label",
        value_name = "TEXT",
        help = "Set the count label for facets (default: preset value)"
    )]
    facets_count_label: Option<String>,
    #[arg(
        long = "files-mode-title",
        value_name = "TEXT",
        help = "Set the files pane title (default: preset value)"
    )]
    files_mode_title: Option<String>,
    #[arg(
        long = "files-hint",
        value_name = "TEXT",
        help = "Set the hint for the files pane (default: preset value)"
    )]
    files_hint: Option<String>,
    #[arg(
        long = "files-table-title",
        value_name = "TEXT",
        help = "Set the table title for files (default: preset value)"
    )]
    files_table_title: Option<String>,
    #[arg(
        long = "files-count-label",
        value_name = "TEXT",
        help = "Set the count label for files (default: preset value)"
    )]
    files_count_label: Option<String>,
    #[arg(
        long = "facet-headers",
        value_delimiter = ',',
        value_name = "HEADER",
        help = "Comma-separated facet table headers (default: preset value)"
    )]
    facet_headers: Option<Vec<String>>,
    #[arg(
        long = "file-headers",
        value_delimiter = ',',
        value_name = "HEADER",
        help = "Comma-separated file table headers (default: preset value)"
    )]
    file_headers: Option<Vec<String>>,
    #[arg(
        short = 'H',
        long = "hidden",
        value_parser = BoolishValueParser::new(),
        help = "Include hidden files (default: enabled)"
    )]
    hidden: Option<bool>,
    #[arg(
        short = 's',
        long = "follow-symlinks",
        value_parser = BoolishValueParser::new(),
        help = "Follow symbolic links while scanning (default: disabled)"
    )]
    follow_symlinks: Option<bool>,
    #[arg(
        long = "respect-ignore-files",
        value_parser = BoolishValueParser::new(),
        help = "Respect .ignore files (default: enabled)"
    )]
    respect_ignore_files: Option<bool>,
    #[arg(
        long = "git-ignore",
        value_parser = BoolishValueParser::new(),
        help = "Respect .gitignore files (default: enabled)"
    )]
    git_ignore: Option<bool>,
    #[arg(
        long = "git-global",
        value_parser = BoolishValueParser::new(),
        help = "Respect global gitignore settings (default: enabled)"
    )]
    git_global: Option<bool>,
    #[arg(
        long = "git-exclude",
        value_parser = BoolishValueParser::new(),
        help = "Respect git exclude files (default: enabled)"
    )]
    git_exclude: Option<bool>,
    #[arg(
        short = 'j',
        long,
        value_name = "NUM",
        help = "Limit the number of indexing threads (default: automatic)"
    )]
    threads: Option<usize>,
    #[arg(
        short = 'd',
        long = "max-depth",
        value_name = "NUM",
        help = "Limit directory traversal depth (default: unlimited)"
    )]
    max_depth: Option<usize>,
    #[arg(
        long = "extensions",
        value_delimiter = ',',
        value_name = "EXT",
        help = "Restrict search to specific file extensions (default: all)"
    )]
    extensions: Option<Vec<String>>,
    #[arg(
        long = "context-label",
        value_name = "TEXT",
        help = "Override the context label shown in the prompt (default: derived from filesystem root)"
    )]
    context_label: Option<String>,
    #[arg(
        long = "global-ignores",
        value_delimiter = ',',
        value_name = "NAME",
        help = "Comma-separated directory names to always ignore (default: .git,node_modules,target,.venv)"
    )]
    global_ignores: Option<Vec<String>>,
    #[arg(
        short = 'p',
        long = "print-config",
        help = "Print the resolved configuration before running (default: disabled)"
    )]
    print_config: bool,
    #[arg(
        short = 'l',
        long = "list-themes",
        help = "List supported themes and exit (default: disabled)"
    )]
    list_themes: bool,
    #[arg(short = 'o', long = "output", value_enum, default_value_t = OutputFormat::Plain, help = "Choose how to print the result")]
    output: OutputFormat,
}

#[cfg(feature = "fs")]
#[derive(Copy, Clone, Debug, ValueEnum)]
enum ModeArg {
    Facets,
    Files,
}

#[cfg(feature = "fs")]
impl ModeArg {
    fn as_str(self) -> &'static str {
        match self {
            ModeArg::Facets => "facets",
            ModeArg::Files => "files",
        }
    }
}

#[cfg(feature = "fs")]
#[derive(Copy, Clone, Debug, ValueEnum)]
enum UiPresetArg {
    Default,
    #[clap(name = "tags-and-files")]
    TagsAndFiles,
}

#[cfg(feature = "fs")]
impl UiPresetArg {
    fn as_str(self) -> &'static str {
        match self {
            UiPresetArg::Default => "default",
            UiPresetArg::TagsAndFiles => "tags-and-files",
        }
    }
}

#[cfg(feature = "fs")]
#[derive(Copy, Clone, Debug, ValueEnum)]
enum OutputFormat {
    Plain,
    Json,
}

#[cfg(feature = "fs")]
fn main() -> Result<()> {
    let cli = parse_cli();

    if cli.list_themes {
        for name in frz::theme::NAMES {
            println!("{name}");
        }
        return Ok(());
    }

    let resolved = settings::load(&cli)?;

    if cli.print_config {
        resolved.print_summary();
    }

    run_search(cli.output, resolved)
}

#[cfg(feature = "fs")]
fn run_search(format: OutputFormat, settings: ResolvedConfig) -> Result<()> {
    let ResolvedConfig {
        root,
        filesystem,
        input_title,
        initial_query,
        theme,
        start_mode,
        ui,
        facet_headers,
        file_headers,
    } = settings;

    let mut searcher = Searcher::filesystem_with_options(root, filesystem)?;

    if let Some(title) = input_title {
        searcher = searcher.with_input_title(title);
    }

    searcher = searcher.with_ui_config(ui);
    searcher = searcher.with_initial_query(initial_query);

    if let Some(theme) = theme {
        searcher = searcher.with_theme_name(&theme);
    }

    if let Some(mode) = start_mode {
        searcher = searcher.with_start_mode(mode);
    }

    if let Some(headers) = facet_headers {
        let refs: Vec<&str> = headers.iter().map(|header| header.as_str()).collect();
        searcher = searcher.with_headers_for(SearchMode::Facets, refs);
    }

    if let Some(headers) = file_headers {
        let refs: Vec<&str> = headers.iter().map(|header| header.as_str()).collect();
        searcher = searcher.with_headers_for(SearchMode::Files, refs);
    }

    let outcome = searcher.run()?;

    match format {
        OutputFormat::Plain => print_plain(&outcome),
        OutputFormat::Json => print_json(&outcome)?,
    }

    Ok(())
}

#[cfg(feature = "fs")]
fn print_plain(outcome: &SearchOutcome) {
    if !outcome.accepted {
        println!("Search cancelled (query: '{}')", outcome.query);
        return;
    }

    match &outcome.selection {
        Some(SearchSelection::File(file)) => println!("{}", file.path),
        Some(SearchSelection::Facet(facet)) => println!("Facet: {}", facet.name),
        None => println!("No selection"),
    }
}

#[cfg(feature = "fs")]
fn print_json(outcome: &SearchOutcome) -> Result<()> {
    let selection = match &outcome.selection {
        Some(SearchSelection::File(file)) => json!({
            "type": "file",
            "path": file.path,
            "tags": file.tags,
            "display_tags": file.display_tags,
        }),
        Some(SearchSelection::Facet(facet)) => json!({
            "type": "facet",
            "name": facet.name,
            "count": facet.count,
        }),
        None => serde_json::Value::Null,
    };

    let payload = json!({
        "accepted": outcome.accepted,
        "query": outcome.query,
        "selection": selection,
    });

    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

#[cfg(not(feature = "fs"))]
fn main() {
    eprintln!("The frz binary requires the 'fs' feature to be enabled.");
}
