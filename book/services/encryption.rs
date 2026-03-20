//! Offline encryption service using Argon2id (KDF) + XChaCha20-Poly1305 (AEAD)
//!
//! Features
//! - Encrypt arbitrary plaintext (e.g., Markdown) with a user password.
//! - Generate a random database key, then wrap (encrypt) that key with a user password.
//! - Unwrap (decrypt) the database key with a password.
//! - Change (rotate) the wrapping password by re-wrapping the key.
//! - Security hygiene: zeroize sensitive data; optional best-effort mlock; strict length checks; AEAD error propagation.
//! - `scheme_version` included for forward migration.
//!
//! NOTE: This service is intended for local-only storing Markdown as encrypted blobs.
//!
//! ## Quick usage
//! ```rust
//! use encryption_service::*;
//!
//! // Encrypt/Decrypt Markdown
//! let password = "correct horse battery staple";
//! let md = "# My Note\nSecret text".as_bytes();
//! let blob_json = encrypt_with_password(password, md)?; // JSON string safe to store
//! let plaintext = decrypt_with_password(password, &blob_json)?; // Vec<u8>
//!
//! // Generate DB key and wrap with password
//! let db_key = generate_db_key(); // Zeroizing<[u8;32]>
//! let wrapped_json = wrap_key_with_password(password, &db_key)?;
//! let unwrapped = unwrap_key_with_password(password, &wrapped_json)?; // Zeroizing<Vec<u8>> (32 bytes)
//!
//! // Change password (re-wrap without exposing the key)
//! let new_wrapped = change_password(&wrapped_json, password, "new-password")?;
//! ```
//!
use argon2::{Argon2, Params};
use cfg_if::cfg_if;
use chacha20poly1305::{
    Key, XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit},
};
use getrandom::{SysRng, rand_core::TryRng};
use hkdf::Hkdf;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use thiserror::Error;
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

// ---------------------- Configuration / Limits ----------------------

/// Scheme version included in envelope for future migration.
const SCHEME_VERSION: u16 = 1;
const AAD_DOMAIN: &str = "encryption_service:v1"; // part of AEAD AAD for domain separation

const MASTER_KEY_LEN: usize = 32;
pub const DERIVED_KEY_LEN: usize = 32; // 256-bit keys
// Defaults chosen for a *local* markdown app. Benchmark on your target devices.
const DEFAULT_ARGON2_MEMORY_KIB: u32 = 65536; // 64 MiB
const DEFAULT_ARGON2_ITERATIONS: u32 = 5; // Will make it slow but more secure
const DEFAULT_ARGON2_PARALLELISM: u32 = 4;

// Maximum limits to avoid OOM & validate inputs (tune as needed)
const MAX_SALT_LEN: usize = 64; // 32..64 bytes
const MIN_SALT_LEN: usize = 32;
const NONCE_LEN: usize = 24; // XChaCha20-Poly1305
const _AEAD_TAG_LEN: usize = 16;
const MAX_PLAINTEXT_LEN: usize = 5 * 1024 * 1024; // 5 MiB per markdown blob
const MAX_CIPHERTEXT_LEN: usize = MAX_PLAINTEXT_LEN + _AEAD_TAG_LEN + 128; // some slack

// ---------------------- Types & Errors ----------------------

/// KDF parameters stored with the ciphertext to allow future migrations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdfParams {
    pub version: u32,
    pub alg: String,     // "argon2id"
    pub m_cost_kib: u32, // memory cost in KiB
    pub t_cost: u32,     // iterations
    pub p_cost: u32,     // parallelism
    #[serde(with = "serde_bytes")]
    pub salt: Vec<u8>, // raw bytes in memory; for JSON envelope we use base64
    pub workspace_id: String,
}

impl KdfParams {
    pub fn new(workspace_id: &str) -> Self {
        let mut salt: Vec<u8> = vec![0u8; MIN_SALT_LEN];
        let _ = SysRng.try_fill_bytes(&mut salt);

        KdfParams {
            version: 1,
            alg: Self::MAIN_ALGORITHM.to_string(),
            m_cost_kib: Self::MEMORY_COST_KIB,
            t_cost: Self::ITERATIONS,
            p_cost: Self::PARALELLISM,
            salt,
            workspace_id: workspace_id.to_string(),
        }
    }
}

impl Default for KdfParams {
    /// Default Argon2id parameters (balanced for offline KDF); tune as desired.
    fn default() -> Self {
        // Generate raw salt bytes directly
        let mut salt: Vec<u8> = vec![0u8; MIN_SALT_LEN];
        let _ = SysRng.try_fill_bytes(&mut salt);

        KdfParams {
            version: 1,
            alg: Self::MAIN_ALGORITHM.to_string(),
            m_cost_kib: Self::MEMORY_COST_KIB,
            t_cost: Self::ITERATIONS,
            p_cost: Self::PARALELLISM,
            salt,
            workspace_id: String::new(),
        }
    }
}

impl KdfParams {
    pub const MEMORY_COST_KIB: u32 = DEFAULT_ARGON2_MEMORY_KIB;
    pub const MAIN_ALGORITHM: &str = "argon2id";
    pub const MAIN_NOTE_ALGORITHM: &str = "hdkf:sha256";
    pub const ITERATIONS: u32 = DEFAULT_ARGON2_ITERATIONS;
    pub const PARALELLISM: u32 = DEFAULT_ARGON2_PARALLELISM;

    fn derive_note_salt(notes_id: &Uuid) -> Vec<u8> {
        format!("mindsafe:noteKey:{notes_id}:v1").into_bytes()
    }

    fn notes_default(notes_id: &Uuid, workspace_id: &str) -> Self {
        KdfParams {
            version: 1,
            alg: Self::MAIN_NOTE_ALGORITHM.to_string(),
            m_cost_kib: 0,
            t_cost: 0,
            p_cost: 0,
            salt: KdfParams::derive_note_salt(notes_id),
            workspace_id: workspace_id.to_string(),
        }
    }

    pub fn get_from_saved(saved_params: &KdfParams) -> Self {
        KdfParams {
            // using standard to ensuring bad params are not supplied
            // in future in case version 2 comes then we can handle this
            version: 1,
            alg: Self::MAIN_ALGORITHM.to_string(),
            m_cost_kib: Self::MEMORY_COST_KIB,
            t_cost: Self::ITERATIONS,
            p_cost: Self::PARALELLISM,
            salt: saved_params.salt.clone(),
            workspace_id: saved_params.workspace_id.clone(),
        }
    }
    pub fn get_from_note(saved_params: &KdfParams) -> Self {
        KdfParams {
            version: 1,
            alg: Self::MAIN_NOTE_ALGORITHM.to_string(),
            m_cost_kib: 0,
            t_cost: 0,
            p_cost: 0,
            salt: saved_params.salt.clone(),
            workspace_id: saved_params.workspace_id.clone(),
        }
    }
}

impl Zeroize for KdfParams {
    fn zeroize(&mut self) {
        self.alg.zeroize();
        self.m_cost_kib.zeroize();
        self.t_cost.zeroize();
        self.p_cost.zeroize();
        self.salt.zeroize();
    }
}

impl ZeroizeOnDrop for KdfParams {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EncryptedBlob {
    pub scheme_version: u16,
    pub kdf: KdfParams,
    #[serde(with = "serde_bytes")]
    pub nonce: Vec<u8>, // 24 bytes for XChaCha20-Poly1305
    #[serde(with = "serde_bytes")]
    pub ciphertext: Vec<u8>,
}

impl Zeroize for EncryptedBlob {
    fn zeroize(&mut self) {
        self.scheme_version.zeroize();
        self.kdf.zeroize();
        self.nonce.zeroize();
        self.ciphertext.zeroize();
    }
}

impl ZeroizeOnDrop for EncryptedBlob {}

impl EncryptedBlob {
    /// Serialize `EncryptedBlob` into a JSON string
    pub fn serialize(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| CryptoError::Serde(e.to_string()))
    }

    /// Deserialize `EncryptedBlob` from a JSON string
    pub fn deserialize(s: &str) -> Result<Self> {
        serde_json::from_str(s).map_err(|e| CryptoError::Serde(e.to_string()))
    }
}

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("invalid input: {0}")]
    InvalidInput(&'static str),
    #[error("kdf error")]
    Kdf,
    #[error("aead error (decryption failure)")]
    Aead,
    #[error("serialization error: {0}")]
    Serde(String),
}

type Result<T> = core::result::Result<T, CryptoError>;

// ---------------------- KDF / Key derivation ----------------------

/// Create Argon2 instance from stored params.
fn argon2_from_params(k: &KdfParams) -> Result<Argon2<'static>> {
    let base_params: KdfParams = KdfParams::get_from_saved(k);
    let params: Params = Params::new(
        base_params.m_cost_kib,
        base_params.t_cost,
        base_params.p_cost,
        Some(32), // output length
    )
    .map_err(|_| CryptoError::Kdf)?;

    Ok(Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        params,
    ))
}

/// Derive 32-byte key from password and KDF params.
fn derive_key_from_bytes(password: &[u8], kdf: &KdfParams) -> Result<Zeroizing<[u8; 32]>> {
    if kdf.salt.len() < MIN_SALT_LEN || kdf.salt.len() > MAX_SALT_LEN {
        return Err(CryptoError::InvalidInput("salt length out of range"));
    }

    let argon2: Argon2<'static> = argon2_from_params(kdf)?;
    let mut key: Zeroizing<[u8; 32]> = Zeroizing::new([0u8; 32]);

    argon2
        .hash_password_into(password, &kdf.salt, &mut *key)
        .map_err(|_| CryptoError::Kdf)?;

    Ok(key)
}

// ---------------------- AAD builder ----------------------

/// Build AAD from fields to ensure tamper-evident header binding.
fn aad_bytes(version: u16, kdf: &KdfParams) -> Result<Vec<u8>> {
    if kdf.salt.len() > (u16::MAX as usize) {
        return Err(CryptoError::InvalidInput("salt too large for AAD"));
    }

    let mut aad = Vec::new();

    // domain
    aad.extend_from_slice(AAD_DOMAIN.as_bytes());

    // fixed-width numeric fields
    aad.extend_from_slice(&version.to_be_bytes());

    // variable-length fields, prefix with u16 length
    let alg_bytes = kdf.alg.as_bytes();
    if alg_bytes.len() > (u16::MAX as usize) {
        return Err(CryptoError::InvalidInput("alg string too long"));
    }
    aad.extend_from_slice(&(alg_bytes.len() as u16).to_be_bytes());
    aad.extend_from_slice(alg_bytes);

    aad.extend_from_slice(&kdf.m_cost_kib.to_be_bytes());
    aad.extend_from_slice(&kdf.t_cost.to_be_bytes());
    aad.extend_from_slice(&kdf.p_cost.to_be_bytes());

    aad.extend_from_slice(&(kdf.salt.len() as u16).to_be_bytes());
    aad.extend_from_slice(&kdf.salt);

    Ok(aad)
}

// ---------------------- Memory locking helpers (best-effort) ----------------------

fn lock_memory(buf: &mut [u8]) {
    #[allow(unused_variables)]
    let ptr = buf.as_mut_ptr() as *mut core::ffi::c_void;
    let _len: usize = buf.len();

    cfg_if! {
        if #[cfg(all(feature = "mlock", unix))] {
            unsafe {
                let _ = libc::mlock(ptr, _len);
            }
        } else if #[cfg(all(feature = "mlock", target_os = "windows"))] {
            unsafe {
                use winapi::um::memoryapi::VirtualLock;
                let _ = VirtualLock(ptr, _len);
            }
        } else {
            // no-op
        }
    }
}

fn unlock_memory(buf: &mut [u8]) {
    #[allow(unused_variables)]
    let ptr = buf.as_mut_ptr() as *mut core::ffi::c_void;
    let _len: usize = buf.len();

    cfg_if! {
        if #[cfg(all(feature = "mlock", unix))] {
            unsafe {
                let _ = libc::munlock(ptr, _len);
            }
        } else if #[cfg(all(feature = "mlock", target_os = "windows"))] {
            unsafe {
                use winapi::um::memoryapi::VirtualUnlock;
                let _ = VirtualUnlock(ptr, _len);
            }
        } else {
            // no-op
        }
    }
}

// ---------------------- Core APIs ----------------------

pub struct EncryptionService {}

impl EncryptionService {
    /// Encrypt arbitrary plaintext with a password (convenience, accepts &str).
    /// Returns compact base64 JSON blob string.
    pub(super) fn encrypt_with_password(
        workspace_id: &str,
        password: &str,
        plaintext: &[u8],
    ) -> Result<EncryptedBlob> {
        let pw_bytes: &[u8] = password.as_bytes();
        encrypt_with_password_bytes(workspace_id, &Zeroizing::new(pw_bytes.to_vec()), plaintext)
    }

    /// Decrypt with password convenience wrapper (accepts &str).
    pub(super) fn decrypt_with_password(password: &str, blob: &EncryptedBlob) -> Result<Vec<u8>> {
        let pw_bytes = password.as_bytes();
        decrypt_with_password_bytes(&Zeroizing::new(pw_bytes.to_vec()), blob)
    }

    /// Generate a 32-byte random database key.
    pub(super) fn generate_master_key() -> Vec<u8> {
        let mut key: Vec<u8> = vec![0u8; MASTER_KEY_LEN];
        let _ = SysRng.try_fill_bytes(&mut key);
        key
    }

    /// Example usage: generate db_key and file_key
    pub fn generate_keys(master_key: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let db_key = derive_key(master_key, b"mindsafe:dbKey:v1");
        let file_key = derive_key(master_key, b"mindsafe:fileKey:v1");

        (db_key, file_key)
    }

    /// Encrypt arbitrary plaintext with a password (convenience, accepts &str).
    /// Returns compact base64 JSON blob string.
    pub(super) fn encrypt_with_key(
        workspace_id: &str,
        file_key: &[u8],
        note_id: &Uuid,
        plaintext: &[u8],
    ) -> Result<EncryptedBlob> {
        encrypt_with_note_key_bytes(
            workspace_id,
            &Zeroizing::new(file_key.to_vec()),
            note_id,
            plaintext,
        )
    }

    /// Decrypt with password convenience wrapper (accepts &str).
    pub(super) fn decrypt_with_key(
        file_key: &[u8],
        note_id: &Uuid,
        blob: &EncryptedBlob,
    ) -> Result<Vec<u8>> {
        decrypt_with_note_key_bytes(&Zeroizing::new(file_key.to_vec()), note_id, blob)
    }

    // Change password for a wrapped key by decrypting and re-encrypting with the new password.
    // pub fn change_password(
    //     wrapped_json: &str,
    //     old_password: &str,
    //     new_password: &str,
    // ) -> Result<String> {
    //     let key = unwrap_key_with_password(old_password, wrapped_json)?; // Zeroized on drop
    //     wrap_key_with_password(new_password, &key)
    // }
}

/// Derive a subkey from the master key with HKDF-SHA256.
/// `info` ensures keys are domain-separated (different purpose = different key).
fn derive_key(master_key: &[u8], info: &[u8]) -> Vec<u8> {
    let hk = Hkdf::<Sha256>::new(None, master_key);
    let mut okm = vec![0u8; DERIVED_KEY_LEN];
    hk.expand(info, &mut okm).expect("HKDF expand failed");
    okm
}

// Encrypt with password bytes so the caller can pass Zeroizing<Vec<u8>> and avoid
/// leaving an extra non-zeroizable copy of the password in memory.
fn encrypt_with_note_key_bytes(
    workspace_id: &str,
    file_key: &Zeroizing<Vec<u8>>,
    note_id: &Uuid,
    plaintext: &[u8],
) -> Result<EncryptedBlob> {
    if plaintext.is_empty() {
        return Err(CryptoError::InvalidInput("empty plaintext"));
    }
    if plaintext.len() > MAX_PLAINTEXT_LEN {
        return Err(CryptoError::InvalidInput("plaintext too large"));
    }

    // Prepare KDF params and derive key
    let kdf: KdfParams = KdfParams::notes_default(note_id, workspace_id);

    // Derive notes key from file_key and info (salt)
    let note_key = derive_key(file_key, &kdf.salt);

    // AEAD setup
    let cipher = {
        let kref: &Key = Key::from_slice(&note_key);
        XChaCha20Poly1305::new(kref)
    };

    // Nonce
    let mut nonce: Vec<u8> = vec![0u8; NONCE_LEN];
    let _ = SysRng.try_fill_bytes(&mut nonce);

    // AAD binds domain + scheme version + serialized KDF header
    let mut aad: Vec<u8> = aad_bytes(SCHEME_VERSION, &kdf)?;

    // Encrypt
    let ct: Vec<u8> = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            chacha20poly1305::aead::Payload {
                msg: plaintext,
                aad: &aad,
            },
        )
        .map_err(|_| CryptoError::Aead)?;

    aad.zeroize();

    // Build base64 envelope
    let encrypted_blob: EncryptedBlob = EncryptedBlob {
        scheme_version: SCHEME_VERSION,
        kdf,
        nonce,
        ciphertext: ct,
    };

    // Serialize as JSON
    // let json: String =
    //     serde_json::to_string(&blob_b64).map_err(|e| CryptoError::Serde(e.to_string()))?;

    // Zeroize ephemeral buffers (blob_b64 holds clones; ensure clones zeroized on drop)
    // Note: JSON string contains base64-encoded data and cannot be zeroized.

    Ok(encrypted_blob)
    // Ok(json)
}

/// Decrypt with password bytes (Zeroizing<Vec<u8>> allows caller to zeroize password memory).
fn decrypt_with_note_key_bytes(
    file_key: &Zeroizing<Vec<u8>>,
    note_id: &Uuid,
    blob: &EncryptedBlob,
) -> Result<Vec<u8>> {
    if blob.scheme_version != SCHEME_VERSION {
        return Err(CryptoError::InvalidInput("unsupported scheme_version"));
    }

    if blob.kdf.salt != KdfParams::derive_note_salt(note_id) {
        return Err(CryptoError::InvalidInput("bad salt value"));
    }
    if blob.kdf.alg != KdfParams::MAIN_NOTE_ALGORITHM {
        return Err(CryptoError::InvalidInput("unsupported kdf"));
    }
    if blob.kdf.m_cost_kib != 0 || blob.kdf.t_cost != 0 || blob.kdf.p_cost != 0 {
        return Err(CryptoError::InvalidInput("bad kdf params"));
    }

    if blob.nonce.len() != NONCE_LEN {
        return Err(CryptoError::InvalidInput("bad nonce length"));
    }

    if blob.ciphertext.len() > MAX_CIPHERTEXT_LEN {
        return Err(CryptoError::InvalidInput("cipher text length too long"));
    }

    let note_key = derive_key(file_key, &blob.kdf.salt);

    // Prepare key
    let cipher = {
        let kref: &Key = Key::from_slice(&note_key);
        XChaCha20Poly1305::new(kref)
    };

    // AAD must match what was used during encryption
    let aad = aad_bytes(SCHEME_VERSION, &blob.kdf)?;

    // Decrypt
    let pt = cipher
        .decrypt(
            XNonce::from_slice(&blob.nonce),
            chacha20poly1305::aead::Payload {
                msg: &blob.ciphertext,
                aad: &aad,
            },
        )
        .map_err(|_| CryptoError::Aead)?;

    Ok(pt)
}

// Encrypt with password bytes so the caller can pass Zeroizing<Vec<u8>> and avoid
/// leaving an extra non-zeroizable copy of the password in memory.
fn encrypt_with_password_bytes(
    workspace_id: &str,
    password: &Zeroizing<Vec<u8>>,
    plaintext: &[u8],
) -> Result<EncryptedBlob> {
    if plaintext.is_empty() {
        return Err(CryptoError::InvalidInput("empty plaintext"));
    }
    if plaintext.len() > MAX_PLAINTEXT_LEN {
        return Err(CryptoError::InvalidInput("plaintext too large"));
    }

    // Prepare KDF params and derive key
    let kdf: KdfParams = KdfParams::new(workspace_id);

    let mut key: Zeroizing<[u8; 32]> = derive_key_from_bytes(password, &kdf)?;
    lock_memory(&mut *key);

    // AEAD setup
    let cipher = {
        let kref: &Key = Key::from_slice(&*key);
        XChaCha20Poly1305::new(kref)
    };

    // Nonce
    let mut nonce: Vec<u8> = vec![0u8; NONCE_LEN];
    let _ = SysRng.try_fill_bytes(&mut nonce);

    // AAD binds domain + scheme version + serialized KDF header
    let mut aad: Vec<u8> = aad_bytes(SCHEME_VERSION, &kdf)?;

    // Encrypt
    let ct: Vec<u8> = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            chacha20poly1305::aead::Payload {
                msg: plaintext,
                aad: &aad,
            },
        )
        .map_err(|_| CryptoError::Aead)?;

    unlock_memory(&mut *key);

    // Build base64 envelope
    let encrypted_blob: EncryptedBlob = EncryptedBlob {
        scheme_version: SCHEME_VERSION,
        kdf,
        nonce,
        ciphertext: ct,
    };

    aad.zeroize();
    key.zeroize();

    // Serialize as JSON
    // let json: String =
    //     serde_json::to_string(&blob_b64).map_err(|e| CryptoError::Serde(e.to_string()))?;

    // Zeroize ephemeral buffers (blob_b64 holds clones; ensure clones zeroized on drop)
    // Note: JSON string contains base64-encoded data and cannot be zeroized.

    Ok(encrypted_blob)
    // Ok(json)
}

/// Decrypt with password bytes (Zeroizing<Vec<u8>> allows caller to zeroize password memory).
fn decrypt_with_password_bytes(
    password: &Zeroizing<Vec<u8>>,
    blob: &EncryptedBlob,
) -> Result<Vec<u8>> {
    if blob.scheme_version != SCHEME_VERSION {
        return Err(CryptoError::InvalidInput("unsupported scheme_version"));
    }

    if blob.kdf.salt.len() < MIN_SALT_LEN || blob.kdf.salt.len() > MAX_SALT_LEN {
        return Err(CryptoError::InvalidInput("salt length out of range"));
    }
    if Uuid::parse_str(&blob.kdf.workspace_id).is_err() {
        return Err(CryptoError::InvalidInput("invalid workspace_id UUID"));
    }
    if blob.kdf.alg != KdfParams::MAIN_ALGORITHM {
        return Err(CryptoError::InvalidInput("unsupported kdf"));
    }
    if blob.kdf.m_cost_kib == 0 || blob.kdf.t_cost == 0 || blob.kdf.p_cost == 0 {
        return Err(CryptoError::InvalidInput("bad kdf params"));
    }

    if blob.nonce.len() != NONCE_LEN {
        return Err(CryptoError::InvalidInput("bad nonce length"));
    }

    if blob.ciphertext.len() > MAX_CIPHERTEXT_LEN {
        return Err(CryptoError::InvalidInput("cipher text length too long"));
    }

    // Prepare key
    let key = derive_key_from_bytes(password, &blob.kdf)?;
    let cipher = {
        let kref: &Key = Key::from_slice(&*key);
        XChaCha20Poly1305::new(kref)
    };

    // AAD must match what was used during encryption
    let aad = aad_bytes(SCHEME_VERSION, &blob.kdf)?;

    // Decrypt
    let pt = cipher
        .decrypt(
            XNonce::from_slice(&blob.nonce),
            chacha20poly1305::aead::Payload {
                msg: &blob.ciphertext,
                aad: &aad,
            },
        )
        .map_err(|_| CryptoError::Aead)?;

    Ok(pt)
}

// ---------------------- Key wrapping helpers ----------------------

// Wrap (encrypt) a key with a password. Returns JSON blob string.
// fn wrap_key_with_password(password: &str, key_material: &[u8]) -> Result<String> {
//     if key_material.len() != 32 {
//         return Err(CryptoError::InvalidInput("key must be 32 bytes"));
//     }
//     let blob_b64 = EncryptionService::encrypt_with_password(password, key_material)?;
//     let json: String =
//         serde_json::to_string(&blob_b64).map_err(|e| CryptoError::Serde(e.to_string()))?;
//     Ok(json)
// }

// Unwrap (decrypt) a key with a password. Returns the raw key.
// fn unwrap_key_with_password(password: &str, wrapped_json: &str) -> Result<Zeroizing<Vec<u8>>> {
//     let blob_b64: EncryptedBlobB64 =
//         serde_json::from_str(wrapped_json).map_err(|e| CryptoError::Serde(e.to_string()))?;
//     let key = EncryptionService::decrypt_with_password(password, &blob_b64)?;
//     if key.len() != 32 {
//         return Err(CryptoError::InvalidInput("wrapped key wrong length"));
//     }
//     Ok(key)
// }

// fn wrap_key_with_password_bytes(
//     password: &Zeroizing<Vec<u8>>,
//     key_material: &[u8],
// ) -> Result<String> {
//     if key_material.len() != 32 {
//         return Err(CryptoError::InvalidInput("key must be 32 bytes"));
//     }
//     encrypt_with_password_bytes(password, key_material)
// }

// fn unwrap_key_with_password_bytes(
//     password: &Zeroizing<Vec<u8>>,
//     wrapped_json: &str,
// ) -> Result<Zeroizing<Vec<u8>>> {
//     let key = decrypt_with_password_bytes(password, wrapped_json)?;
//     if key.len() != 32 {
//         return Err(CryptoError::InvalidInput("wrapped key wrong length"));
//     }
//     Ok(key)
// }

// fn change_password_bytes(
//     wrapped_json: &str,
//     old_password: &Zeroizing<Vec<u8>>,
//     new_password: &Zeroizing<Vec<u8>>,
// ) -> Result<String> {
//     let key = unwrap_key_with_password_bytes(old_password, wrapped_json)?; // Zeroized on drop
//     wrap_key_with_password_bytes(new_password, &key)
// }
