# riz

TUI fuzzy finder revolving around tabular data, utilising [Saghen](https://github.com/Saghen)'s [Frizbee](https://github.com/Saghen/frizbee) crate for matching.

## Features
- Interactive TUI built on `ratatui`.
- Uses `frizbee` fuzzy matching for typo-tolerant search.
- Builder-style API to configure prompts, column headers and widths.
- Ready-to-use filesystem scanner (`Searcher::filesystem`) that walks directories recursively.
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

## Run the examples

```bash
cargo run -p riz --example demo
cargo run -p riz --example filesystem -- /path/to/project
```
