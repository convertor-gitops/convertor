//! Stateless Plan transport encoding used by subscription URLs.

use super::Plan;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use flate2::{Compression, read::DeflateDecoder, write::DeflateEncoder};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

const MAGIC: &[u8; 3] = b"CVP";
const CODEC_VERSION: u8 = 1;

/// Hard limit for the encoded `plan` query value.
pub const MAX_ENCODED_PLAN_BYTES: usize = 64 * 1024;
/// Hard limit for JSON produced by decompression.
pub const MAX_PLAN_JSON_BYTES: usize = 1024 * 1024;
/// URL length above which the API warns about client and proxy compatibility.
pub const PLAN_URL_WARNING_BYTES: usize = 8 * 1024;

/// Stable digest of the canonical compact JSON representation of a Plan.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PlanFingerprint(pub String);

/// Encoded transport value together with the semantic Plan fingerprint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncodedPlan {
    pub value: String,
    pub fingerprint: PlanFingerprint,
}

/// Failures while validating, encoding or decoding a Plan URL payload.
#[derive(Debug, thiserror::Error)]
pub enum PlanCodecError {
    #[error("invalid plan: {0}")]
    InvalidPlan(String),
    #[error("plan JSON serialization failed")]
    Serialize(#[source] serde_json::Error),
    #[error("plan JSON is too large")]
    JsonTooLarge,
    #[error("encoded plan is too large")]
    EncodedTooLarge,
    #[error("invalid plan encoding")]
    InvalidBase64,
    #[error("invalid plan envelope")]
    InvalidEnvelope,
    #[error("unsupported plan codec version {0}")]
    UnsupportedCodecVersion(u8),
    #[error("invalid compressed plan")]
    InvalidCompression,
    #[error("plan JSON is not valid UTF-8")]
    InvalidUtf8,
    #[error("invalid plan JSON")]
    InvalidJson(#[source] serde_json::Error),
}

/// Encode a validated Plan as `CVP + version + raw DEFLATE + Base64URL`.
pub fn encode_plan(plan: &Plan) -> Result<EncodedPlan, PlanCodecError> {
    validate(plan)?;
    let json = canonical_json(plan)?;
    if json.len() > MAX_PLAN_JSON_BYTES {
        return Err(PlanCodecError::JsonTooLarge);
    }

    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&json).map_err(|_| PlanCodecError::InvalidCompression)?;
    let compressed = encoder.finish().map_err(|_| PlanCodecError::InvalidCompression)?;

    let mut envelope = Vec::with_capacity(MAGIC.len() + 1 + compressed.len());
    envelope.extend_from_slice(MAGIC);
    envelope.push(CODEC_VERSION);
    envelope.extend_from_slice(&compressed);
    let value = URL_SAFE_NO_PAD.encode(envelope);
    if value.len() > MAX_ENCODED_PLAN_BYTES {
        return Err(PlanCodecError::EncodedTooLarge);
    }

    Ok(EncodedPlan {
        value,
        fingerprint: fingerprint_bytes(&json),
    })
}

/// Decode and fully validate a Plan query value.
pub fn decode_plan(value: &str) -> Result<Plan, PlanCodecError> {
    if value.len() > MAX_ENCODED_PLAN_BYTES {
        return Err(PlanCodecError::EncodedTooLarge);
    }
    let envelope = URL_SAFE_NO_PAD.decode(value).map_err(|_| PlanCodecError::InvalidBase64)?;
    if envelope.len() < 4 || &envelope[..3] != MAGIC {
        return Err(PlanCodecError::InvalidEnvelope);
    }
    if envelope[3] != CODEC_VERSION {
        return Err(PlanCodecError::UnsupportedCodecVersion(envelope[3]));
    }

    let decoder = DeflateDecoder::new(&envelope[4..]);
    let mut json = Vec::new();
    decoder
        .take((MAX_PLAN_JSON_BYTES + 1) as u64)
        .read_to_end(&mut json)
        .map_err(|_| PlanCodecError::InvalidCompression)?;
    if json.len() > MAX_PLAN_JSON_BYTES {
        return Err(PlanCodecError::JsonTooLarge);
    }
    std::str::from_utf8(&json).map_err(|_| PlanCodecError::InvalidUtf8)?;
    let plan: Plan = serde_json::from_slice(&json).map_err(PlanCodecError::InvalidJson)?;
    validate(&plan)?;
    Ok(plan)
}

/// Fingerprint a Plan independently of its compressed representation.
pub fn fingerprint_plan(plan: &Plan) -> Result<PlanFingerprint, PlanCodecError> {
    canonical_json(plan).map(|json| fingerprint_bytes(&json))
}

fn canonical_json(plan: &Plan) -> Result<Vec<u8>, PlanCodecError> {
    serde_json::to_vec(plan).map_err(PlanCodecError::Serialize)
}

fn fingerprint_bytes(json: &[u8]) -> PlanFingerprint {
    PlanFingerprint(blake3::hash(json).to_hex().to_string())
}

fn validate(plan: &Plan) -> Result<(), PlanCodecError> {
    plan.validate().map_err(|errors| PlanCodecError::InvalidPlan(errors.join("; ")))
}
