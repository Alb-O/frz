use clap::ValueEnum;

/// Search modes accepted via the command line.
#[derive(Copy, Clone, Debug, ValueEnum)]
pub(crate) enum ModeArg {
    Attributes,
    Files,
}

impl ModeArg {
    /// Return the string representation consumed by configuration loading.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ModeArg::Attributes => "attributes",
            ModeArg::Files => "files",
        }
    }
}

/// Predefined UI presets selectable from the CLI.
#[derive(Copy, Clone, Debug, ValueEnum)]
pub(crate) enum UiPresetArg {
    Default,
    #[clap(name = "tags-and-files")]
    TagsAndFiles,
}

impl UiPresetArg {
    /// Return the preset identifier consumed by configuration loading.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            UiPresetArg::Default => "default",
            UiPresetArg::TagsAndFiles => "tags-and-files",
        }
    }
}

/// Output formats supported by the CLI utility.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum OutputFormat {
    Plain,
    Json,
}
