use std::fmt;

#[derive(Debug, Clone)]
pub(crate) enum SettingSource {
	CliFlag(&'static str),
	Environment(&'static str),
	ConfigKey(&'static str),
}

impl fmt::Display for SettingSource {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::CliFlag(flag) => write!(f, "CLI flag `{flag}`"),
			Self::Environment(var) => write!(f, "environment variable `{var}`"),
			Self::ConfigKey(key) => write!(f, "configuration key `{key}`"),
		}
	}
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ConfigSources {
	pub(crate) filesystem_threads: Option<SettingSource>,
	pub(crate) filesystem_max_depth: Option<SettingSource>,
}

impl ConfigSources {
	pub(crate) fn source_for_threads(&self) -> SettingSource {
		self.filesystem_threads
			.clone()
			.unwrap_or(SettingSource::ConfigKey("filesystem.threads"))
	}

	pub(crate) fn source_for_max_depth(&self) -> SettingSource {
		self.filesystem_max_depth
			.clone()
			.unwrap_or(SettingSource::ConfigKey("filesystem.max_depth"))
	}
}
