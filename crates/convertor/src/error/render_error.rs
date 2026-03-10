use crate::error::InternalError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("[Renderer] 渲染失败: {0}")]
    Render(String),

    #[error("[Renderer] 渲染失败")]
    Unknown(#[from] Box<InternalError>),
}

impl From<std::fmt::Error> for RenderError {
    fn from(err: std::fmt::Error) -> Self {
        RenderError::Unknown(Box::new(InternalError::Fmt(err)))
    }
}
