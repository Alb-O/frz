# -- UNDER CONSTRUCTION --

# frz

TUI fuzzy finder revolving around tabular data, utilising [Saghen](https://github.com/Saghen)'s [Frizbee](https://github.com/Saghen/frizbee) crate for matching.

## Features
- Interactive TUI built on `ratatui`.
- Uses `frizbee` fuzzy matching for typo-tolerant search.
- Builder-style API to configure prompts, column headers and widths.
- Ready-to-use filesystem scanner (`SearchUi::filesystem`) that walks directories recursively.
- Multi-threaded filesystem traversal powered by the [`ignore`](https://docs.rs/ignore) crate with built-in `.gitignore` support.
- Rich outcome information including which entry was selected and the final query string.

## Architecture

frz is split into three layers: infrastructure systems, extensions, and the TUI
application. `src/extensions/api/` defines the stable extension surface, including
descriptors and the `ExtensionModule` trait. `crates/tui/` offers reusable widgets
and helpers for rendering extension output. The binary crate in `src/` wires these
pieces together, initialises background systems, and registers built-in extensions
via [`register_builtin_extensions`](src/extensions/builtin/mod.rs).

## Quick example

```rust
use frz::{SearchData, SearchUi, UiConfig};
use frz::extensions::builtin::files;

let data = SearchData::from_filesystem(".")?;
let outcome = SearchUi::new(data)
    .with_ui_config(UiConfig::tags_and_files())
    .run()?;

if let Some(file) = outcome.selected_file() {
    println!("Selected file: {}", file.path);
}
```

## Run the examples

```bash
cargo run -p frz --example demo
cargo run -p frz --example filesystem -- /path/to/project
```

## Command-line application and configuration

The crate now ships with a `frz` binary that provides a ready-to-use filesystem
search experience. You can explore the available options with:

```bash
cargo run -- --help
```

frz loads configuration from a layered set of sources:

1. `~/.config/frz/config.toml` (or the platform-specific directory reported by
   [`directories::ProjectDirs`](https://docs.rs/directories)).
2. `$FRZ_CONFIG_DIR/config.toml` if the environment variable is set.
3. `./.frz.toml` followed by `./frz.toml` in the current working directory.
4. Any files passed via `--config <path>` (later files win).
5. Environment variables prefixed with `FRZ_` using `__` as a separator
   (for example `FRZ_FILESYSTEM__INCLUDE_HIDDEN=false`).
6. Explicit command-line flags.

A minimal configuration might look like this:

```toml
[filesystem]
root = "~/projects/frz"
include_hidden = false
allowed_extensions = ["rs", "toml"]

[ui]
theme = "solarized"
detail_panel_title = "Entry details"
```

You can inspect the resolved configuration before launching the TUI via
`--print-config`, list available themes with `--list-themes`, or emit the final
selection as pretty JSON using `--output json`.

## Extending via extensions

- Extensions can register new tabs by implementing [`ExtensionModule`](https://docs.rs/frz/latest/frz/trait.ExtensionModule.html) and adding them to an [`ExtensionCatalog`](https://docs.rs/frz/latest/frz/struct.ExtensionCatalog.html). Each module exposes an [`ExtensionDescriptor`](https://docs.rs/frz/latest/frz/extensions/descriptors/struct.ExtensionDescriptor.html) that advertises UI copy, table layout metadata, and an associated [`ExtensionDataset`](https://docs.rs/frz/latest/frz/extensions/descriptors/trait.ExtensionDataset.html) implementation; the dataset abstraction lets modules describe how to render their tables, report aggregate counts, and contribute progress information, enabling the catalog to treat every extension uniformly regardless of how many are registered.
- Catalogs preserve insertion order, making it easy to deterministically compose built-in and custom tabs. They also expose helpers such as [`ExtensionCatalog::remove`](https://docs.rs/frz/latest/frz/struct.ExtensionCatalog.html#method.remove) and [`ExtensionCatalog::module_by_id`](https://docs.rs/frz/latest/frz/struct.ExtensionCatalog.html#method.module_by_id) so applications can swap out built-in implementations or target modules by their identifier without having to manage bookkeeping themselves.
- Reusable background services live under the `extensions::systems` module. The search worker is available via [`extensions::systems::search`](https://docs.rs/frz/latest/frz/extensions/systems/search/), which exposes the [`SearchStream`](https://docs.rs/frz/latest/frz/extensions/systems/search/struct.SearchStream.html) type and helpers for streaming attributes and files using the built-in matching pipeline. The filesystem indexer is exposed through [`extensions::systems::filesystem`](https://docs.rs/frz/latest/frz/extensions/systems/filesystem/), which provides [`FilesystemOptions`](https://docs.rs/frz/latest/frz/extensions/systems/filesystem/struct.FilesystemOptions.html), [`spawn_filesystem_index`](https://docs.rs/frz/latest/frz/extensions/systems/filesystem/fn.spawn_filesystem_index.html), and the [`merge_update`](https://docs.rs/frz/latest/frz/extensions/systems/filesystem/fn.merge_update.html) helper for applying incremental results to `SearchData`.

### Contribution stores

Extensions can provide optional capabilities beyond search tabs by installing
contributions while registering with the catalog:

- [`SearchTabStore`](src/extensions/api/contributions/search_tabs.rs) accepts
  `ExtensionModule` implementations for new tabs.
- [`PreviewSplitStore`](src/extensions/api/contributions/preview_split.rs)
  holds preview renderers that receive a [`PreviewSplitContext`](src/extensions/api/contributions/preview_split.rs#L13-L94)
  describing the current query, filtered rows, and the active selection.
- [`SelectionResolverStore`](src/extensions/api/contributions/selection.rs)
  lets extensions translate the UI selection into a typed
  [`PreviewResource`](src/extensions/api/contributions/selection.rs#L8-L15) so previewers can work with rich domain data instead of table indices.
- [`IconStore`](src/extensions/api/contributions/icons.rs) enables custom icon
  providers for rows rendered in the results table.

Each store exposes a `resolve(mode)` helper and automatically cleans up when a
mode is removed from the catalog, ensuring contribution lifecycles remain in
sync with registered modules.
