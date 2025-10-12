# -- UNDER CONSTRUCTION --

# riz

TUI fuzzy finder revolving around tabular data, utilising [Saghen](https://github.com/Saghen)'s [Frizbee](https://github.com/Saghen/frizbee) crate for matching.

## Features
- Interactive TUI built on `ratatui`.
- Uses `frizbee` fuzzy matching for typo-tolerant search.
- Builder-style API to configure prompts, column headers and widths.
- Ready-to-use filesystem scanner (`Searcher::filesystem`) that walks directories recursively.
- Multi-threaded filesystem traversal powered by the [`ignore`](https://docs.rs/ignore) crate with built-in `.gitignore` support.
- Rich outcome information including which entry was selected and the final query string.

## Quick example

```rust
use riz::{SearchData, SearchMode, Searcher, UiConfig};

let data = SearchData::from_filesystem(".")?;
let outcome = Searcher::new(data)
    .with_ui_config(UiConfig::tags_and_files())
    .with_start_mode(SearchMode::Files)
    .run()?;

if let Some(file) = outcome.selected_file() {
    println!("Selected file: {}", file.path);
}
```

> **Note:** Filesystem helpers are gated behind the default-enabled `fs` feature. Disable default features or omit `fs` when you
> want to build riz without any filesystem access.

## Run the examples

```bash
cargo run -p riz --example demo
cargo run -p riz --example filesystem -- /path/to/project
```

## Command-line application and configuration

The crate now ships with a `riz` binary that provides a ready-to-use filesystem
search experience. You can explore the available options with:

```bash
cargo run -- --help
```

Riz loads configuration from a layered set of sources:

1. `~/.config/riz/config.toml` (or the platform-specific directory reported by
   [`directories::ProjectDirs`](https://docs.rs/directories)).
2. `$RIZ_CONFIG_DIR/config.toml` if the environment variable is set.
3. `./.riz.toml` followed by `./riz.toml` in the current working directory.
4. Any files passed via `--config <path>` (later files win).
5. Environment variables prefixed with `RIZ_` using `__` as a separator
   (for example `RIZ_FILESYSTEM__INCLUDE_HIDDEN=false`).
6. Explicit command-line flags.

A minimal configuration might look like this:

```toml
[filesystem]
root = "~/projects/riz"
include_hidden = false
allowed_extensions = ["rs", "toml"]

[ui]
theme = "solarized"
start_mode = "files"
detail_panel_title = "Entry details"
```

You can inspect the resolved configuration before launching the TUI via
`--print-config`, list available themes with `--list-themes`, or emit the final
selection as pretty JSON using `--output json`.
