// use crate::error::EncryptError;
// use base64::Engine;
// use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64URL;
// use chacha20poly1305::aead::{Aead, KeyInit};
// use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
// use rand_core::OsRng;
// use std::cell::RefCell;
//
// type Result<T> = core::result::Result<T, EncryptError>;
//
// // ===== 线程局部：给“当前线程”注入可复现的 RNG =====
// // 每个测试线程可在第一行设置自己的种子，互不影响，支持并行。
// thread_local! {
//     static TL_SEEDED_RNG: RefCell<Option<rand_chacha::ChaCha20Rng>> = const { RefCell::new(None) };
// }
//
// /// 在当前线程启用“固定种子”的伪随机数源（可复现，适合快照）
// pub fn nonce_rng_use_seed(seed: [u8; 32]) {
//     use rand_core::SeedableRng;
//     TL_SEEDED_RNG.with(|c| *c.borrow_mut() = Some(rand_chacha::ChaCha20Rng::from_seed(seed)));
// }
//
// /// 在当前线程恢复为系统 RNG（生产默认行为）
// pub fn nonce_rng_use_system() {
//     TL_SEEDED_RNG.with(|c| *c.borrow_mut() = None);
// }
//
// /// 统一生成 24B nonce：优先线程局部 RNG，缺省回退 OS RNG
// fn gen_nonce24() -> Result<[u8; 24]> {
//     // 1) 先试线程局部的“固定种子” RNG（可复现、并行互不影响）
//     if let Some(n) = TL_SEEDED_RNG.with(|cell| {
//         let mut opt = cell.borrow_mut();
//         if let Some(rng) = opt.as_mut() {
//             use rand_core::RngCore; // infallible
//             let mut n = [0u8; 24];
//             rng.fill_bytes(&mut n);
//             Some(n)
//         } else {
//             None
//         }
//     }) {
//         return Ok(n);
//     }
//
//     // 2) 否则使用 OS RNG（rand_core 0.9 里 OsRng 实现 TryRngCore）
//     let mut n = [0u8; 24];
//     {
//         use rand_core::TryRngCore;
//         let mut rng = OsRng;
//         rng.try_fill_bytes(&mut n)?;
//     }
//     Ok(n)
// }
//
// const NONCE_LEN: usize = 24;
// const NONCE_B64URL_LEN: usize = 32; // 24 bytes -> 32 chars (url-safe, no pad)
//
// fn normalize_key(key: &[u8]) -> [u8; 32] {
//     let mut normalized = [0u8; 32];
//     let len = key.len().min(32);
//     normalized[..len].copy_from_slice(&key[..len]);
//     normalized
// }
//
// pub fn encrypt(secret: &[u8], plaintext: &str) -> Result<String> {
//     let norm_key = normalize_key(secret);
//     let key = Key::from_slice(&norm_key);
//     let cipher = XChaCha20Poly1305::new(key);
//
//     // 统一从线程局部/OsRng 取 nonce
//     let nonce_bytes = gen_nonce24()?;
//     let nonce = XNonce::from_slice(&nonce_bytes);
//
//     let ciphertext = cipher
//         .encrypt(nonce, plaintext.as_bytes())
//         .map_err(|_| EncryptError::Encrypt)?;
//
//     // URL-safe, no padding；不加任何分隔符
//     let mut out = String::with_capacity(NONCE_B64URL_LEN + (ciphertext.len() * 4).div_ceil(3));
//     out.push_str(&B64URL.encode(nonce));
//     out.push_str(&B64URL.encode(ciphertext));
//     Ok(out)
// }
//
// pub fn decrypt(secret: &[u8], token: &str) -> Result<String> {
//     if token.len() < NONCE_B64URL_LEN {
//         return Err(EncryptError::NonceLength);
//     }
//     let (nonce_part, ct_part) = token.split_at(NONCE_B64URL_LEN);
//
//     // 先解 nonce
//     let nonce_raw = B64URL.decode(nonce_part).map_err(EncryptError::DecodeError)?;
//     if nonce_raw.len() != NONCE_LEN {
//         return Err(EncryptError::NonceLength);
//     }
//     let nonce = XNonce::from_slice(&nonce_raw);
//
//     // 再解密文
//     let ciphertext = B64URL.decode(ct_part).map_err(EncryptError::DecodeError)?;
//
//     let norm_key = normalize_key(secret);
//     let key = Key::from_slice(&norm_key);
//     let cipher = XChaCha20Poly1305::new(key);
//
//     let plaintext = cipher
//         .decrypt(nonce, ciphertext.as_ref())
//         .map_err(|_| EncryptError::Decrypt)?;
//
//     Ok(String::from_utf8(plaintext)?)
// }

use crate::error::EncryptError;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use rand_chacha::ChaCha20Rng;
use rand_core::{Rng, SeedableRng};

type Result<T> = core::result::Result<T, EncryptError>;

const NONCE_LEN: usize = 24;
const NONCE_B64URL_LEN: usize = 32; // 24 bytes -> 32 chars (base64url no pad)

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seed(pub [u8; 32]);

impl Seed {
    pub fn random() -> Result<Self> {
        let mut s = [0u8; 32];
        let mut rng = rand_core::SeedableRng;
        rng.fill_bytes(&mut s);
        Ok(Seed(s))
    }

    pub fn to_b64url(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.0)
    }

    pub fn from_b64url(s: &str) -> Result<Self> {
        let bytes = URL_SAFE_NO_PAD
            .decode(s.as_bytes())
            .map_err(|e| anyhow!("invalid seed b64url: {e}"))?;
        if bytes.len() != 32 {
            return Err(anyhow!("seed must be 32 bytes, got {}", bytes.len()));
        }
        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        Ok(Seed(out))
    }
}

enum NonceSource {
    Random,                     // OsRng
    Deterministic(ChaCha20Rng), // seeded, reproducible stream
}

pub struct Encryptor {
    cipher: XChaCha20Poly1305,
    nonce_src: NonceSource,
}

impl Encryptor {
    /// 生产默认：随机 nonce（推荐）
    pub fn new_random(secret: &[u8]) -> Result<Self> {
        let key_bytes = normalize_key_32(secret);
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&key_bytes));
        Ok(Self {
            cipher,
            nonce_src: NonceSource::Random,
        })
    }

    /// 测试/可复现：固定 seed（注意不要在生产长期复用）
    pub fn new_with_seed(secret: &[u8], seed: Seed) -> Result<Self> {
        let key_bytes = normalize_key_32(secret);
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&key_bytes));
        let rng = ChaCha20Rng::from_seed(seed.0);
        Ok(Self {
            cipher,
            nonce_src: NonceSource::Deterministic(rng),
        })
    }

    /// 生成 seed（用于持久化）
    pub fn gen_seed() -> Result<Seed> {
        Seed::random()
    }

    pub fn encrypt(&mut self, plaintext: &str) -> Result<String> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        match &mut self.nonce_src {
            NonceSource::Random => rand_core::OsRng.fill_bytes(&mut nonce_bytes),
            NonceSource::Deterministic(rng) => rng.fill_bytes(&mut nonce_bytes),
        }

        let nonce = XNonce::from_slice(&nonce_bytes);
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|_| anyhow!("encrypt failed"))?;

        // token = b64url(nonce) || b64url(ciphertext)
        let mut out = String::with_capacity(NONCE_B64URL_LEN + (ciphertext.len() * 4).div_ceil(3));
        out.push_str(&URL_SAFE_NO_PAD.encode(nonce_bytes));
        out.push_str(&URL_SAFE_NO_PAD.encode(ciphertext));
        Ok(out)
    }

    /// 解密不依赖 seed
    pub fn decrypt(&self, token: &str) -> Result<String> {
        if token.len() < NONCE_B64URL_LEN {
            return Err(anyhow!("token too short"));
        }
        let (nonce_b64, ct_b64) = token.split_at(NONCE_B64URL_LEN);

        let nonce_vec = URL_SAFE_NO_PAD
            .decode(nonce_b64.as_bytes())
            .map_err(|e| anyhow!("invalid nonce b64url: {e}"))?;
        if nonce_vec.len() != NONCE_LEN {
            return Err(anyhow!("nonce must be 24 bytes, got {}", nonce_vec.len()));
        }
        let mut nonce_bytes = [0u8; NONCE_LEN];
        nonce_bytes.copy_from_slice(&nonce_vec);

        let ciphertext = URL_SAFE_NO_PAD
            .decode(ct_b64.as_bytes())
            .map_err(|e| anyhow!("invalid ciphertext b64url: {e}"))?;

        let nonce = XNonce::from_slice(&nonce_bytes);
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| anyhow!("decrypt failed"))?;

        Ok(String::from_utf8(plaintext)?)
    }
}

/// 你自己的 normalize_key：最终必须得到 32 bytes key
fn normalize_key_32(secret: &[u8]) -> [u8; 32] {
    // 示例：如果 secret 已经是 32 bytes 就直接用；否则做 KDF/HKDF/Blake3 派生
    use blake3::Hasher;
    let mut h = Hasher::new();
    h.update(secret);
    let out = h.finalize();
    *out.as_bytes()
}
