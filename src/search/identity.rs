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
