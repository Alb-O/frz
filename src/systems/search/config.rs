use frizbee::Options;

use super::PREFILTER_ENABLE_THRESHOLD;

/// Builds fuzzy matching options for the provided query and dataset size.
pub(crate) fn config_for_query(query: &str, dataset_len: usize) -> Options {
    let mut config = Options {
        prefilter: false,
        ..Options::default()
    };

    let length = query.chars().count();
    let mut allowed_typos: u16 = match length {
        0 => 0,
        1 => 0,
        2..=4 => 1,
        5..=7 => 2,
        8..=12 => 3,
        _ => 4,
    };
    if let Ok(max_reasonable) = u16::try_from(length.saturating_sub(1)) {
        allowed_typos = allowed_typos.min(max_reasonable);
    }

    if dataset_len >= PREFILTER_ENABLE_THRESHOLD {
        config.prefilter = true;
        config.max_typos = Some(allowed_typos);
    } else {
        config.prefilter = false;
        config.max_typos = None;
    }

    config.sort = false;

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enables_prefilter_for_large_datasets() {
        let config = config_for_query("example", PREFILTER_ENABLE_THRESHOLD);
        assert!(config.prefilter);
        assert_eq!(config.max_typos, Some(2));
    }

    #[test]
    fn disables_prefilter_for_small_datasets() {
        let config = config_for_query("example", PREFILTER_ENABLE_THRESHOLD - 1);
        assert!(!config.prefilter);
        assert_eq!(config.max_typos, None);
    }
}
