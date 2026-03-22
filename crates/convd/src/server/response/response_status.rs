use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ResponseStatus {
    pub code: u64,
    pub status: &'static str,
}

pub const Ok: ResponseStatus = ResponseStatus { code: 0, status: "ok" };
pub const Error: ResponseStatus = ResponseStatus { code: 1, status: "error" };

impl Default for ResponseStatus {
    fn default() -> Self {
        Self { code: 0, status: "ok" }
    }
}

impl ResponseStatus {
    pub fn ok() -> Self {
        Self { code: 0, status: "ok" }
    }
}
