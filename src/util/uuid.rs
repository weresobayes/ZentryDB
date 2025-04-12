use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use uuid::Uuid;

/// Generate a deterministic UUID from a hashable value.
/// The same input will always generate the same UUID.
/// 
/// # Implementation
/// Uses a 64-bit hash of the input to generate a 128-bit UUID:
/// - First 8 bytes: Direct from hash
/// - Last 8 bytes: Bit-rotated hash for better distribution
pub fn generate_deterministic_uuid<T: Hash>(value: &T) -> Uuid {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    let hash = hasher.finish();
    
    let mut uuid_bytes = [0u8; 16];
    uuid_bytes[0..8].copy_from_slice(&hash.to_le_bytes());
    uuid_bytes[8..16].copy_from_slice(&(hash.rotate_right(32)).to_le_bytes());
    
    Uuid::from_bytes(uuid_bytes)
}
