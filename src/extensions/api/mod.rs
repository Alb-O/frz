pub mod context;
pub mod contributions;
pub mod descriptors;
pub mod error;
pub mod registry;
pub mod search;
pub mod streams;

pub use context::{ExtensionQueryContext, ExtensionSelectionContext};
pub use contributions::{
    Contribution, ContributionScope, ContributionStores, ExtensionPackage, Icon, IconProvider,
    IconResource, IconStore, PreviewResource, PreviewSplit, PreviewSplitContext, PreviewSplitStore,
    ScopedContribution, SelectionResolver, SelectionResolverStore,
};
pub use descriptors::{
    ExtensionDataset, ExtensionDescriptor, ExtensionUiDefinition, TableContext, TableDescriptor,
};
pub use error::ExtensionCatalogError;
pub use registry::{ExtensionCatalog, ExtensionModule, RegisteredModule};
pub use search::{
    AttributeRow, ExtensionSelection, FileRow, MAX_RENDERED_RESULTS, MatchBatch,
    PREFILTER_ENABLE_THRESHOLD, SearchData, SearchMode, SearchOutcome, SearchResult,
    SearchSelection, SearchStream, SearchView, SearchViewV2, TruncationStyle, stream_attributes,
    stream_files, tags_for_relative_path,
};
pub use streams::{DataStream, StreamAction, StreamEnvelope, ViewAction, ViewTarget};
