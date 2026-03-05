use thiserror::Error;

#[derive(Debug, Error)]
pub enum EncryptError {
    #[error("无法分离 nonce 与密文")]
    SplitError,

    // 你现在 decrypt() 里用它表示 token 太短/nonce 长度不对
    #[error("nonce 长度不合法")]
    NonceLength,

    #[error("加密失败")]
    Encrypt,

    #[error("解密失败")]
    Decrypt,

    #[error("解码 base64 字符串失败")]
    Decode(#[from] base64::DecodeError),

    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),

    // rand_core::OsError 已经不适配你现在的实现；随机数来自 getrandom
    #[error(transparent)]
    Rng(#[from] getrandom::Error),
}
