use sha2::{Digest, Sha256};

/// Computes the lowercase hex-encoded SHA-256 digest of any string.
/// Used consistently across forgot_password and reset_password so
/// the same hash algorithm is always applied in both places.
pub fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}
