use crate::error::EncryptError;
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

fn normalize_key_32(key: &[u8]) -> [u8; 32] {
    // 兼容你旧逻辑：截断/补零（不是 KDF）
    let mut normalized = [0u8; 32];
    let len = key.len().min(32);
    normalized[..len].copy_from_slice(&key[..len]);
    normalized
}

fn seed_from_label(label: &str) -> [u8; 32] {
    *blake3::hash(label.as_bytes()).as_bytes()
}

fn nonce_from_seed_cursor(seed32: &[u8; 32], cursor: u64) -> [u8; NONCE_LEN] {
    let mut hasher = blake3::Hasher::new_keyed(seed32);
    hasher.update(&cursor.to_le_bytes());
    let out = hasher.finalize(); // 32 bytes
    let mut nonce = [0u8; NONCE_LEN];
    nonce.copy_from_slice(&out.as_bytes()[..NONCE_LEN]);
    nonce
}

/// 用于 serde 的“纯数据”表示（不含 Cell）
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
enum NonceModeRepr {
    Random,
    Deterministic { seed: [u8; 32], cursor: u64 },
}

#[derive(Debug)]
enum NonceMode {
    Random,
    Deterministic { seed: [u8; 32], cursor: AtomicU64 },
}

impl NonceMode {
    fn to_repr(&self) -> NonceModeRepr {
        match self {
            NonceMode::Random => NonceModeRepr::Random,
            NonceMode::Deterministic { seed, cursor } => NonceModeRepr::Deterministic {
                seed: *seed,
                cursor: cursor.load(Ordering::Relaxed),
            },
        }
    }

    fn from_repr(r: NonceModeRepr) -> Self {
        match r {
            NonceModeRepr::Random => NonceMode::Random,
            NonceModeRepr::Deterministic { seed, cursor } => NonceMode::Deterministic {
                seed,
                cursor: AtomicU64::new(cursor),
            },
        }
    }
}

/// 你需要的那个“可 Debug/Clone/Eq/Hash/serde”的 Encryptor
pub struct Encryptor {
    key: [u8; 32],
    nonce: NonceMode,
}

impl Encryptor {
    pub fn new_random(secret: impl AsRef<str>) -> Self {
        Self {
            key: normalize_key_32(secret.as_ref().as_bytes()),
            nonce: NonceMode::Random,
        }
    }

    pub fn new_with_label(secret: impl AsRef<str>, label: impl AsRef<str>) -> Self {
        Self {
            key: normalize_key_32(secret.as_ref().as_bytes()),
            nonce: NonceMode::Deterministic {
                seed: seed_from_label(label.as_ref()),
                cursor: AtomicU64::new(0),
            },
        }
    }

    pub fn new_with_label_and_cursor(secret: impl AsRef<str>, label: impl AsRef<str>, cursor: u64) -> Self {
        Self {
            key: normalize_key_32(secret.as_ref().as_bytes()),
            nonce: NonceMode::Deterministic {
                seed: seed_from_label(label.as_ref()),
                cursor: AtomicU64::new(cursor),
            },
        }
    }

    #[inline]
    fn cipher(&self) -> XChaCha20Poly1305 {
        XChaCha20Poly1305::new(Key::from_slice(&self.key))
    }

    fn gen_nonce24(&self) -> Result<[u8; NONCE_LEN]> {
        match &self.nonce {
            NonceMode::Random => {
                let mut n = [0u8; NONCE_LEN];
                getrandom::fill(&mut n)?;
                Ok(n)
            }
            NonceMode::Deterministic { seed, cursor } => {
                let c = cursor.fetch_add(1, Ordering::Relaxed);
                Ok(nonce_from_seed_cursor(seed, c))
            }
        }
    }

    /// token = b64url(nonce24) || b64url(ciphertext_with_tag)
    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let nonce_bytes = self.gen_nonce24()?;
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

        let nonce_raw = B64URL.decode(nonce_part)?;
        if nonce_raw.len() != NONCE_LEN {
            return Err(EncryptError::NonceLength);
        }
        let nonce = XNonce::from_slice(&nonce_raw);

        let ciphertext = B64URL.decode(ct_part)?;

        let plaintext = self
            .cipher()
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| EncryptError::Decrypt)?;

        Ok(String::from_utf8(plaintext)?)
    }
}

/* ===== 手动实现你要求的 trait：Debug/Clone/Eq/PartialEq/Hash/Serialize/Deserialize ===== */

impl Debug for Encryptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // key 只显示前 4 bytes + 长度，避免泄露
        let key_prefix = &self.key[..4];

        f.debug_struct("Encryptor")
            .field("key_prefix", &key_prefix)
            .field("key_len", &self.key.len())
            .field("nonce", &self.nonce.to_repr()) // 用 repr，避免 Cell 细节
            .finish()
    }
}

impl Clone for Encryptor {
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            nonce: NonceMode::from_repr(self.nonce.to_repr()),
        }
    }
}

impl PartialEq for Encryptor {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.nonce.to_repr() == other.nonce.to_repr()
    }
}
impl Eq for Encryptor {}

impl Hash for Encryptor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        self.nonce.to_repr().hash(state);
    }
}

impl Serialize for Encryptor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Repr {
            key: [u8; 32],
            nonce: NonceModeRepr,
        }
        let r = Repr {
            key: self.key,
            nonce: self.nonce.to_repr(),
        };
        r.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Encryptor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> core::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Repr {
            key: [u8; 32],
            nonce: NonceModeRepr,
        }
        let r = Repr::deserialize(deserializer)?;
        Ok(Self {
            key: r.key,
            nonce: NonceMode::from_repr(r.nonce),
        })
    }
}
