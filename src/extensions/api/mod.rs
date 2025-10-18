pub mod context;
pub mod contributions;
pub mod descriptors;
pub mod error;
pub mod registry;
pub mod search;

pub use context::{ExtensionQueryContext, ExtensionSelectionContext};
pub use contributions::{Contribution, ExtensionPackage, PreviewSplit, PreviewSplitContext};
pub use descriptors::{
    ExtensionDataset, ExtensionDescriptor, ExtensionUiDefinition, TableContext, TableDescriptor,
};
pub use error::ExtensionCatalogError;
pub use registry::{ExtensionCatalog, ExtensionModule, RegisteredModule};
pub use search::{
    AttributeRow, ExtensionSelection, FileRow, MAX_RENDERED_RESULTS, PREFILTER_ENABLE_THRESHOLD,
    SearchData, SearchMode, SearchOutcome, SearchResult, SearchSelection, SearchStream,
    TruncationStyle, stream_attributes, stream_files, tags_for_relative_path,
};
