use frizbee::Options;

/// Tunable thresholds shared across the search pipeline.
pub const PREFILTER_ENABLE_THRESHOLD: usize = 1_000;

/// Maximum number of rows rendered in the result table.
pub const MAX_RENDERED_RESULTS: usize = 2_000;

/// Number of matches processed per scoring chunk.
pub const MATCH_CHUNK_SIZE: usize = 512;

/// Number of rows processed before emitting a heartbeat for empty queries.
pub const EMPTY_QUERY_BATCH: usize = 128;

/// Compute a stable 64-bit hash for the provided value.
///
/// This uses a simple FNV-1a implementation to avoid pulling in
/// additional dependencies while guaranteeing deterministic output across
/// processes and platforms.
#[must_use]
pub fn stable_hash64(value: &str) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x00000100000001b3;

    let mut hash = FNV_OFFSET;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Builds fuzzy matching options for the provided query and dataset size.
pub fn config_for_query(query: &str, dataset_len: usize) -> Options {
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
