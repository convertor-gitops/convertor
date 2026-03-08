use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("渲染失败: {0}")]
    Render(String),

    #[error(transparent)]
    FmtError(#[from] std::fmt::Error),

    #[error(transparent)]
    YamlError(#[from] serde_yml::Error),
}
