# frz Architecture Notes: Data Flow, Orthogonality, and Modularity

This document captures an opinionated review of the current data flow and API design across the frz codebase, along with suggested patterns and targeted refactors to maximize orthogonality, decoupling, and testability. It includes sketches, snippets, and concrete task lists.


## Overview: Core Flows

- UI runtime
  - `src/ui/state.rs` owns `App`, which holds `SearchData`, tab state, and the active `ExtensionCatalog`.
  - `App` spawns the background search worker with a cloned `SearchData` and catalog: `src/systems/search/worker.rs`.
  - The UI implements view traits that receive streamed actions and apply them to state.

- Search streaming
  - Extensions implement `ExtensionModule` (src/extensions/api/registry/module.rs) with `stream` and `selection`.
  - The worker wraps an `mpsc::Sender` into a `SearchStream` (src/extensions/api/search/stream.rs), which is a thin adapter over a generic `DataStream` (src/extensions/api/streams/mod.rs).
  - Producers emit typed actions via `StreamEnvelope<kind, ViewAction<T>>` and the UI dispatches them (`result.dispatch(self)` in `src/ui/search.rs`).

- Filesystem indexer
  - The indexer streams `IndexResult` (src/systems/filesystem/stream.rs), using the same envelope/action pattern.
  - The UI applies updates to its `SearchData` and forwards a mirrored update to the search worker so its copy stays in sync (`SearchRuntime::notify_of_update`).

- Contributions
  - Extensions register contributions in stores keyed by `SearchMode` via a central `ContributionRegistry` (src/extensions/api/contributions/mod.rs).
  - Current contributions: search tabs (`SearchTabStore`), preview split renderers (`PreviewSplitStore`), and icons (`IconStore`). Stores provide clean `resolve(mode)` access and register cleanup hooks for removal.


## Sketch: Data and Message Flow (ASCII)

```
+---------------------+                 +---------------------------+
|         UI          |  SearchCommand  |        Search Worker      |
|  (App<'_>)          | ───────────────▶| (ExtensionCatalog + Data) |
|  - implements       |                 | - module.stream(query)    |
|    SearchView       |  SearchResult   | - SearchStream(DataStream)|
|    IndexView        | ◀───────────────|                           |
+---------┬-----------+                 +---------------------------+
          │ dispatch(ViewAction)                          ▲
          │                                               │ Update(StreamAction<SearchData>)
          ▼                                               │
  apply to App state                                      │
                                                          │
              +------------------- Filesystem Indexer -------------------+
              |  IndexStream(IndexResult) via DataStream                 |
              |  - UI implements IndexView                               |
              |  - UI applies IndexUpdate to SearchData and forwards to  |
              |    Search worker via SearchRuntime::notify_of_update     |
              +----------------------------------------------------------+
```


## What’s Working Well

- Unified streaming pattern
  - `DataStream`, `StreamEnvelope`, and `ViewAction` form a clean, reusable cross-thread messaging abstraction. Search and indexing share this machinery.
- Extension registry and contribution stores
  - Stores keyed by `SearchMode` with cleanup hooks are simple, composable, and support isolation of responsibilities (tabs, icons, preview splits).
- Clear separation of roles
  - UI owns presentation and runtime orchestration; worker owns query execution; extensions own domain behavior.
- Batched streaming with aggregation
  - `ScoreAggregator` and `AlphabeticalCollector` minimize UI jitter and overhead while preserving responsiveness.


## Pain Points and Coupling to Improve

- Index-based selection coupling
  - Search/UI communicate selected rows by positional indices into `SearchData`. Index merges can shift positions, risking drift or extra coordination.
- Dataset-specific duplication
  - `stream_attributes` and `stream_files` are near clones with different key extractors.
  - `AlphabeticalCollector` and `ScoreAggregator` are specialized but could be generalized over a dataset key extractor.
- Preview split generality
  - `PreviewSplitContext` exposes `selected_row_index()` and data slices. It works for files but does not directly model “selected resource” across future datasets.
- Descriptor identity via pointer equality
  - `SearchMode` equality and hashing depend on descriptor pointer identity, which is fine for `&'static` constants but constrains future composition/testing.
- Mixed ownership of `SearchData`
  - Both UI and worker hold copies and synchronize via mirrored updates. This works well, but clearer identity semantics per-row would make updates more robust.


## Design Principles to Target

- Stable identities across boundaries
  - Prefer stable row IDs over positional indices when streaming or selecting, so UI state is resilient to concurrent data ingest.
- Dataset-agnostic streaming
  - Build streaming on “how to get a searchable key for index i” and “how to sort/render,” not on concrete dataset types.
- Capability contributions per mode
  - Continue leaning on contribution stores; add new, mode-scoped capabilities (e.g., selection resolvers, preview resolvers) rather than widening single traits.
- Narrow, testable traits
  - Keep traits small and injectable (e.g., dataset key providers, renderers). Favor adapters over inheritance.


## Proposals and Snippets

### 1) Stable Row Identity (opt-in, backward-compatible)

Introduce optional stable IDs for rows to decouple selection from vector indices. Maintain legacy index paths for rendering-only.

- FileRow/AttributeRow
  - Add an optional `id: u64` derived from a stable key (e.g., hash of the normalized path or attribute name).
- Selection and streaming
  - Add new APIs that can send `(ids, scores)` while preserving existing `(indices, scores)` calls. The UI maps IDs to current indices at dispatch time.

Sketch:

```rust
// src/extensions/api/search/stream.rs (new type alongside existing)
#[derive(Clone)]
pub struct MatchBatch {
    pub indices: Vec<usize>,
    pub ids: Option<Vec<u64>>, // new, optional
    pub scores: Vec<u16>,
}

impl<'a> SearchStream<'a> {
    pub fn send_batch(&self, batch: MatchBatch, complete: bool) -> bool {
        let mode = self.mode();
        self.send_with(move |view| {
            // Prefer new path if the view supports it; otherwise fallback.
            if let Some(view2) = (view as &mut dyn std::any::Any)
                .downcast_mut::<dyn crate::ui::traits::SearchViewV2>() {
                view2.replace_matches_v2(mode, batch);
            } else {
                view.replace_matches(mode, batch.indices, batch.scores);
            }
            view.record_completion(mode, complete);
        }, complete)
    }
}
```

UI keeps a map of `id -> current_index` per mode, updated on index ingest. This lets selection remain stable even when vectors reorder.


### 2) Dataset-Agnostic Streaming Helpers

Generalize streaming over a key provider, collapsing `stream_attributes` and `stream_files` into a single helper.

```rust
pub trait Dataset {
    fn len(&self) -> usize;
    fn key_for(&self, index: usize) -> &str; // search key
}

impl Dataset for [AttributeRow] {
    fn len(&self) -> usize { self.len() }
    fn key_for(&self, i: usize) -> &str { &self[i].name }
}

impl Dataset for [FileRow] {
    fn len(&self) -> usize { self.len() }
    fn key_for(&self, i: usize) -> &str { self[i].search_text() }
}

pub fn stream_dataset<D: Dataset>(
    dataset: &D,
    query: &str,
    stream: SearchStream<'_>,
    latest_query_id: &AtomicU64,
) -> bool { /* reuse aggregator/alphabetical with closures */ }
```

This removes duplication, centralizes batch sizing and abort checks, and improves orthogonality.


### 3) Preview Split Context via Resolvers

Rather than making `PreviewSplit` generic (which conflicts with trait objects across FFI-like boundaries), keep `PreviewSplit` as-is but add a mode-scoped selection resolver contribution that converts `(data, filtered, selected)` into a resource reference for previewers.

- New contribution: `SelectionResolverStore`
  - `fn resolve(&self, data: &SearchData, filtered: &[usize], selected: Option<usize>) -> Option<PreviewResource>`
  - `PreviewResource` can be an enum with well-known types (e.g., File, Attribute), or a boxed trait with render-time downcasting. Start with enums for internal modes.

Sketch:

```rust
pub enum PreviewResource<'a> {
    File(&'a FileRow),
    Attribute(&'a AttributeRow),
}

pub trait SelectionResolver: Send + Sync {
    fn resolve(&self, data: &SearchData, filtered: &[usize], selected: Option<usize>)
        -> Option<PreviewResource<'_>>;
}

#[derive(Default, Clone)]
pub struct SelectionResolverStore { /* mode -> resolver */ }
```

`PreviewSplitContext` gets a `resolver: Option<Arc<dyn SelectionResolver>>`. File previewers can rely on the `File` variant; other extensions can provide their own resolvers.


### 4) Make `StreamEnvelope` Consumption Uniform

Today, consumers call `dispatch(view)` directly. Consider a simple adapter API to decouple the UI layer from envelope internals and unify progress/completion handling hooks.

```rust
pub trait EnvelopeSink<T: ?Sized> {
    fn apply(&mut self, action: StreamAction<T>);
}

impl<T: ?Sized> EnvelopeSink<T> for T {
    fn apply(&mut self, action: StreamAction<T>) { action.apply(self) }
}

// call site
let mut view = /* App */;
let env: StreamEnvelope<_, StreamAction<_>> = rx.recv()?;
view.apply(env.payload);
```

This isn’t strictly necessary, but it makes testing simpler and narrows how much the UI needs to know about envelopes.


### 5) Widen Theme-to-Preview Integration

Expose `bat_theme` resolution more explicitly at `App` construction time and thread it via `PreviewSplitContext` (already present) as a first-class context property. Consider a small trait (`SyntaxThemeProvider`) for future non-bat renderers.


## Small Opportunities and Cleanup

- Prefer explicit `newtype` wrappers for identifiers where equalities matter (e.g., `SearchModeId(&'static str)`) for tests/mocks, while retaining `SearchMode` for stores and maps.
- Extract shared constants (batch sizes, thresholds) into a single tuning module. Currently `MATCH_CHUNK_SIZE`, `EMPTY_QUERY_BATCH`, etc., are scattered under search.
- Standardize how “complete” is interpreted and propagated. Both search and index flows use it; document and validate via tests.


## Risks and Migration Notes

- Stable ID introduction
  - Keep existing `(indices, scores)` path for compatibility. Fill ID maps opportunistically; only use them where needed (selection stability, cross-view linking).
- Generic dataset streaming
  - Changes internal helper signatures only; extension-facing APIs stay the same if we keep `ExtensionModule::stream` as-is.
- Selection resolvers
  - Internal-only initially for built-in modes to avoid breaking external modules. Can be documented and stabilized later.


## Task Backlog (Markdown Checklists)

### Identity and Selection
- [ ] Add optional `id: u64` to `FileRow` and `AttributeRow` keyed by stable content.
- [ ] Maintain per-mode `id -> index` map in `App` updated by index ingest.
- [ ] Introduce `MatchBatch` and `SearchStream::send_batch` (keep existing `send`).
- [ ] Add optional `SearchViewV2` that consumes `MatchBatch`; default to legacy.

### Dataset-Agnostic Streaming
- [ ] Introduce `Dataset` trait with `len` and `key_for(index)`.
- [ ] Implement `Dataset` for `[AttributeRow]` and `[FileRow]`.
- [ ] Replace `stream_attributes`/`stream_files` with `stream_dataset` and thin wrappers.
- [ ] Generalize `AlphabeticalCollector` over a key function (already half-there).

### Preview Splits
- [ ] Add `SelectionResolver` and `SelectionResolverStore` contribution.
- [ ] Wire resolver into `PreviewSplitContext` as optional.
- [ ] Implement a file resolver and update `FilePreviewer` to use it when available.

### Streaming and Envelope UX
- [ ] Add a tiny `EnvelopeSink` adapter for easier testing of action execution.
- [ ] Consolidate batch size/threshold constants into a config/tuning module.
- [ ] Clarify and test “complete” semantics across search and index paths.

### Documentation/Tests
- [ ] Add a README section for extension authors explaining contribution types and stores.
- [ ] Unit tests for ID mapping under index updates (ensure selection stability).
- [ ] Benchmarks for dataset-agnostic streaming vs current split helpers.


## References (Key Files)
- Search streaming: src/extensions/api/search/stream.rs, src/extensions/api/streams/mod.rs
- Search helpers: src/extensions/api/search/streaming.rs, src/extensions/api/search/aggregator.rs, src/extensions/api/search/alphabetical.rs
- Extension registry: src/extensions/api/registry/{catalog.rs,module.rs,registered_module.rs}
- Contributions: src/extensions/api/contributions/{mod.rs,icons.rs,preview_split.rs,search_tabs.rs}
- UI runtime: src/ui/{state.rs,search.rs,render.rs,indexing.rs}
- Filesystem worker: src/systems/filesystem/{mod.rs,stream.rs,fs/*.rs}


## Closing Thoughts

The current design is already thoughtfully decoupled: generic streaming envelopes, trait-based views, and contribution stores are strong foundations. The proposals above focus on making data identities stable, reducing dataset-specific duplication, and enriching preview/selection pathways without widening public traits. Adopting them incrementally should keep external module surfaces stable while improving orthogonality and long-term maintainability.

