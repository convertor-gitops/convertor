use super::ExtraFields;
use serde::{Deserialize, Serialize};

/// 一个可由 Surge 和 Mihomo 共同表达的代理节点。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Proxy {
    /// 配置中的节点名称。
    pub name: String,
    /// 标准化的小写协议名，例如 `trojan`、`ss`、`socks5`。
    pub protocol: String,
    /// 远端服务器地址。
    pub server: String,
    /// 远端服务器端口。
    pub port: u16,
    /// 认证密码。`None` 表示原配置未声明，区别于显式空字符串。
    pub password: Option<String>,
    /// 加密套件；仅在对应协议支持时有意义。
    pub cipher: Option<String>,
    /// TLS SNI。
    pub sni: Option<String>,
    /// 是否启用 UDP。
    pub udp: Option<bool>,
    /// 是否启用 TCP Fast Open。
    pub tfo: Option<bool>,
    /// 是否跳过证书校验。
    pub skip_cert_verify: Option<bool>,
    /// Plan 标注产生或原配置携带的节点标签。
    pub tags: Vec<String>,
    /// 当前公共模型未显式建模的同格式节点参数。
    pub extra: ExtraFields,
    /// 节点行尾注释。
    pub comment: Option<String>,
}

impl Proxy {
    /// 当前旧转换内核能够安全处理的协议集合。
    pub fn supports_protocol(s: &str) -> bool {
        crate::core::legacy::profile::proxy::Proxy::supports_protocol(s)
    }
}
