# Architecture

frz is organised as a small workspace with three primary layers: infrastructure
systems, user-facing plugins, and the terminal UI application. Each layer owns a
specific set of responsibilities so changes can happen independently while
keeping the plugin boundary stable.

## Workspace layout

- `crates/plugin-api/` defines the stable surface that third-party plugins
  depend on. It exposes descriptors, registry helpers, streaming primitives,
  and data types that encapsulate fuzzy-matching results.
- `crates/tui/` packages reusable widgets, themes, and utilities for building
  the application interface. It sits on top of the plugin API, providing common
  affordances for rendering search tabs and tables.
- `src/` is the binary crate that wires everything together. It owns startup and
  configuration, initialises background systems, registers built-in plugins, and
  runs the TUI runtime.

## Layers

### Systems

Systems provide long-lived infrastructure such as the filesystem indexer and the
search worker. They live under `src/systems/` and expose functionality that both
built-in and external plugins can reuse. Systems are focused on data production
and mutation: they maintain indices, stream updates, and handle IO. They do not
render UI directly and have no knowledge of tabs or widgets.

### Plugins

Plugins build on top of systems to deliver user-facing experiences. A plugin is
an implementation of `SearchPlugin` from `crates/plugin-api` alongside a static
`SearchPluginDescriptor`. Plugins describe how to stream search results, how to
select items, and how to render tables through dataset implementations. Built-in
plugins are provided by dedicated crates (`frz-plugin-files`,
`frz-plugin-facets`) that are re-exported from `src/plugins/builtin/` so the app
crate can register them without special casing.

To keep plugin registration ergonomic the binary crate uses
`register_builtin_plugins` (`src/plugins/builtin/mod.rs`) during startup. The
registry remains order-preserving, so applications can add or replace plugins
without losing determinism.

### UI

The TUI runtime (under `crates/tui/` and `src/ui/`) consumes plugins and systems
to render interactive tabs. It manages event handling, draws frames via
`ratatui`, and keeps per-frame state minimal to preserve responsiveness even
under heavy indexing load. UI code depends on plugin descriptors to know which
modes to surface and on systems to fetch incremental data updates.

## Extending the layers

Most new functionality starts in the systems layer (introducing a capability) or
in the plugin layer (presenting the capability to users). The UI layer focuses on
rendering and interaction patterns. By keeping the plugin API narrow and stable,
third-party authors can build on frz without adopting the entire workspace.
