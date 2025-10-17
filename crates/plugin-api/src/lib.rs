#[path = "../capabilities/mod.rs"]
pub mod capabilities;
pub mod context;
pub mod descriptors;
pub mod error;
pub mod registry;
pub mod search;

pub use capabilities::{Capability, PluginBundle, PreviewSplit, PreviewSplitContext};
pub use context::{PluginQueryContext, PluginSelectionContext};
pub use descriptors::{
    SearchPluginDataset, SearchPluginDescriptor, SearchPluginUiDefinition, TableContext,
    TableDescriptor,
};
pub use error::PluginRegistryError;
pub use registry::{RegisteredPlugin, SearchPlugin, SearchPluginRegistry};
pub use search::{
    AttributeRow, FileRow, MAX_RENDERED_RESULTS, PREFILTER_ENABLE_THRESHOLD, PluginSelection,
    SearchData, SearchMode, SearchOutcome, SearchResult, SearchSelection, SearchStream,
    TruncationStyle, stream_attributes, stream_files, tags_for_relative_path,
};
