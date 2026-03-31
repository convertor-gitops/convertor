use crate::error::{EncryptError, InternalError};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64URL;
use blake3;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use core::hash::{Hash, Hasher};
use core::sync::atomic::{AtomicU64, Ordering};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

const NONCE_LEN: usize = 24;
const NONCE_B64URL_LEN: usize = 32;

pub type Result<T> = core::result::Result<T, EncryptError>;

fn derive_key_from_secret(secret: &[u8]) -> [u8; 32] {
    *blake3::hash(secret).as_bytes()
}

fn derive_nonce_seed_from_label(label: &str) -> [u8; 32] {
    *blake3::hash(label.as_bytes()).as_bytes()
}

fn derive_nonce_from_seed_and_counter(seed: &[u8; 32], counter: u64) -> [u8; NONCE_LEN] {
    let mut hasher = blake3::Hasher::new_keyed(seed);
    hasher.update(&counter.to_le_bytes());
    let out = hasher.finalize();
    let mut nonce = [0u8; NONCE_LEN];
    nonce.copy_from_slice(&out.as_bytes()[..NONCE_LEN]);
    nonce
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
enum NonceStateSnapshot {
    Random,
    Deterministic { seed: [u8; 32], counter: u64 },
}

#[derive(Debug)]
enum NonceState {
    Random,
    Deterministic { seed: [u8; 32], counter: AtomicU64 },
}

impl NonceState {
    fn as_snapshot(&self) -> NonceStateSnapshot {
        match self {
            NonceState::Random => NonceStateSnapshot::Random,
            NonceState::Deterministic { seed, counter } => NonceStateSnapshot::Deterministic {
                seed: *seed,
                counter: counter.load(Ordering::Relaxed),
            },
        }
    }

    fn from_snapshot(snapshot: NonceStateSnapshot) -> Self {
        match snapshot {
            NonceStateSnapshot::Random => NonceState::Random,
            NonceStateSnapshot::Deterministic { seed, counter } => NonceState::Deterministic {
                seed,
                counter: AtomicU64::new(counter),
            },
        }
    }
}

pub struct Encryptor {
    key: [u8; 32],
    nonce_state: NonceState,
}

impl Encryptor {
    pub fn new_random(secret: impl AsRef<str>) -> Self {
        Self {
            key: derive_key_from_secret(secret.as_ref().as_bytes()),
            nonce_state: NonceState::Random,
        }
    }

    pub fn new_with_label(secret: impl AsRef<str>, label: impl AsRef<str>) -> Self {
        Self {
            key: derive_key_from_secret(secret.as_ref().as_bytes()),
            nonce_state: NonceState::Deterministic {
                seed: derive_nonce_seed_from_label(label.as_ref()),
                counter: AtomicU64::new(0),
            },
        }
    }

    pub fn new_with_label_and_cursor(secret: impl AsRef<str>, label: impl AsRef<str>, cursor: u64) -> Self {
        Self {
            key: derive_key_from_secret(secret.as_ref().as_bytes()),
            nonce_state: NonceState::Deterministic {
                seed: derive_nonce_seed_from_label(label.as_ref()),
                counter: AtomicU64::new(cursor),
            },
        }
    }

    #[inline]
    fn cipher(&self) -> XChaCha20Poly1305 {
        XChaCha20Poly1305::new(Key::from_slice(&self.key))
    }

    fn generate_nonce(&self) -> Result<[u8; NONCE_LEN]> {
        match &self.nonce_state {
            NonceState::Random => {
                let mut nonce = [0u8; NONCE_LEN];
                getrandom::fill(&mut nonce)
                    .map_err(InternalError::Rng)
                    .map_err(Box::new)
                    .map_err(EncryptError::Unknown)?;
                Ok(nonce)
            }
            NonceState::Deterministic { seed, counter } => {
                let next_counter = counter.fetch_add(1, Ordering::Relaxed);
                Ok(derive_nonce_from_seed_and_counter(seed, next_counter))
            }
        }
    }

    /// token = b64url(nonce24) || b64url(ciphertext_with_tag)
    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let nonce_bytes = self.generate_nonce()?;
        let nonce = XNonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher()
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|_| EncryptError::Encrypt)?;

        let mut out = String::with_capacity(NONCE_B64URL_LEN + (ciphertext.len() * 4).div_ceil(3));
        out.push_str(&B64URL.encode(nonce_bytes));
        out.push_str(&B64URL.encode(ciphertext));
        Ok(out)
    }

    pub fn decrypt(&self, token: &str) -> Result<String> {
        if token.len() < NONCE_B64URL_LEN {
            return Err(EncryptError::Split);
        }
        let (nonce_part, ct_part) = token.split_at(NONCE_B64URL_LEN);

        let nonce_raw = B64URL.decode(nonce_part).map_err(EncryptError::B64Decode)?;
        if nonce_raw.len() != NONCE_LEN {
            return Err(EncryptError::NonceLength);
        }
        let nonce = XNonce::from_slice(&nonce_raw);

        let ciphertext = B64URL.decode(ct_part).map_err(EncryptError::B64Decode)?;

        let plaintext = self
            .cipher()
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| EncryptError::Decrypt)?;

        String::from_utf8(plaintext).map_err(EncryptError::CipherUtf8)
    }
}

impl Debug for Encryptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let key_prefix = &self.key[..4];

        f.debug_struct("Encryptor")
            .field("key_prefix", &key_prefix)
            .field("key_len", &self.key.len())
            .field("nonce_state", &self.nonce_state.as_snapshot())
            .finish()
    }
}

impl Clone for Encryptor {
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            nonce_state: NonceState::from_snapshot(self.nonce_state.as_snapshot()),
        }
    }
}

impl PartialEq for Encryptor {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.nonce_state.as_snapshot() == other.nonce_state.as_snapshot()
    }
}
impl Eq for Encryptor {}

impl Hash for Encryptor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        self.nonce_state.as_snapshot().hash(state);
    }
}

impl Serialize for Encryptor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Repr {
            key: [u8; 32],
            nonce_state: NonceStateSnapshot,
        }
        let r = Repr {
            key: self.key,
            nonce_state: self.nonce_state.as_snapshot(),
        };
        r.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Encryptor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> core::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Repr {
            key: [u8; 32],
            nonce_state: NonceStateSnapshot,
        }
        let r = Repr::deserialize(deserializer)?;
        Ok(Self {
            key: r.key,
            nonce_state: NonceState::from_snapshot(r.nonce_state),
        })
    }
}
