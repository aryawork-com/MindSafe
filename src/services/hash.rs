use ::sha2::Digest;
use sha2::Sha256;

pub struct HashService {}

impl HashService {
    pub fn generate_hash(text: &String) -> String {
        // Convert the input text to bytes
        let text_bytes = text.as_bytes();

        // Compute the SHA256 digest
        let hash = Sha256::digest(text_bytes);

        // Convert the digest into a hexadecimal string
        format!("{hash:x}")
    }

    pub fn compare_hashes(hash1: &String, hash2: &String) -> bool {
        hash1 == hash2
    }
}
