pub mod context;
pub mod descriptors;
pub mod registry;
pub mod search;
pub mod types;

pub use context::{PluginQueryContext, PluginSelectionContext};
pub use descriptors::{
    SearchPluginDataset, SearchPluginDescriptor, SearchPluginUiDefinition, TableContext,
    TableDescriptor,
};
pub use registry::{RegisteredPlugin, SearchPlugin, SearchPluginRegistry};
pub use search::{
    MAX_RENDERED_RESULTS, PREFILTER_ENABLE_THRESHOLD, SearchResult, SearchStream,
    stream_attributes, stream_files,
};
pub use types::tags_for_relative_path;
pub use types::{
    AttributeRow, FileRow, PluginSelection, SearchData, SearchMode, SearchOutcome, SearchSelection,
    TruncationStyle,
};
