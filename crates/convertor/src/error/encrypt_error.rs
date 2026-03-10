use crate::error::InternalError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EncryptError {
    #[error("无法分离 nonce 与密文")]
    Split,

    #[error("[Encryptor] nonce 长度不合法")]
    NonceLength,

    #[error("[Encryptor] 加密失败")]
    Encrypt,

    #[error("[Encryptor] 解密失败")]
    Decrypt,

    #[error("[Encryptor] 解码 base64 字符串失败")]
    B64Decode(#[source] base64::DecodeError),

    #[error("[Encryptor] 密文非法 UTF-8")]
    CipherUtf8(#[source] std::string::FromUtf8Error),

    #[error("[Encryptor] 未知错误")]
    Unknown(#[from] Box<InternalError>),
}
