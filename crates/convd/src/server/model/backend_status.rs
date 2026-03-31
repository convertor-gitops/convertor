use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BackendStatus {
    /// 后端版本号，取自 env!("CARGO_PKG_VERSION")
    pub version: String,
    /// 各子服务健康状态
    pub services: Vec<ServiceStatus>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceStatus {
    /// 服务名称，如 "redis", "loki", "tempo"
    pub name: String,
    /// 是否健康
    pub healthy: bool,
    /// 附加信息（错误原因等），健康时可为 None
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl ServiceStatus {
    pub fn healthy(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            healthy: true,
            message: None,
        }
    }

    pub fn unhealthy(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            healthy: false,
            message: Some(message.into()),
        }
    }
}
