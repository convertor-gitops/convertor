use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use convertor::{
    config::proxy_client::ProxyClient,
    core::plan::{
        Builtin, Output, Plan, PlanCodecError, Source, SourceId, SourceInput, Target, decode_plan, encode_plan, fingerprint_plan,
    },
};
use flate2::{Compression, write::DeflateEncoder};
use std::io::Write;

fn plan(content: String) -> Plan {
    Plan {
        version: 1,
        client: ProxyClient::Surge,
        sources: vec![Source {
            id: SourceId(1),
            name: "source".into(),
            input: SourceInput::Inline { content },
            annotations: vec![],
            node_filter: None,
        }],
        grouping_policies: vec![],
        groups: vec![],
        rules: vec![],
        output: Output {
            roots: vec![],
            extra_nodes: vec![],
            fallback: Target::Builtin(Builtin::Direct),
            settings_source: SourceId(1),
        },
    }
}

#[test]
fn plan_codec_is_deterministic_and_roundtrips() {
    let plan = plan("[Proxy]\n".into());
    let first = encode_plan(&plan).unwrap();
    let second = encode_plan(&plan).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.fingerprint, fingerprint_plan(&plan).unwrap());
    assert_eq!(
        serde_json::to_value(decode_plan(&first.value).unwrap()).unwrap(),
        serde_json::to_value(plan).unwrap()
    );
}

#[test]
fn plan_codec_rejects_invalid_envelopes_and_oversized_json() {
    assert!(matches!(decode_plan("not-base64!"), Err(PlanCodecError::InvalidBase64)));
    assert!(matches!(
        decode_plan(&"a".repeat(64 * 1024 + 1)),
        Err(PlanCodecError::EncodedTooLarge)
    ));

    let mut unsupported = URL_SAFE_NO_PAD
        .decode(encode_plan(&plan("[Proxy]\n".into())).unwrap().value)
        .unwrap();
    unsupported[3] = 2;
    assert!(matches!(
        decode_plan(&URL_SAFE_NO_PAD.encode(unsupported)),
        Err(PlanCodecError::UnsupportedCodecVersion(2))
    ));
    assert!(matches!(
        decode_plan(&URL_SAFE_NO_PAD.encode(b"CVP\x01not-a-deflate-stream")),
        Err(PlanCodecError::InvalidCompression)
    ));
    assert!(matches!(
        decode_plan(&envelope(&vec![b'x'; 1024 * 1024 + 1])),
        Err(PlanCodecError::JsonTooLarge)
    ));
    assert!(matches!(decode_plan(&envelope(&[0xff])), Err(PlanCodecError::InvalidUtf8)));
    assert!(matches!(decode_plan(&envelope(b"not json")), Err(PlanCodecError::InvalidJson(_))));

    let oversized = plan("x".repeat(1024 * 1024 + 1));
    assert!(matches!(encode_plan(&oversized), Err(PlanCodecError::JsonTooLarge)));

    let mut invalid_version = plan("[Proxy]\n".into());
    invalid_version.version = 2;
    assert!(fingerprint_plan(&invalid_version).is_ok());
    assert!(matches!(encode_plan(&invalid_version), Err(PlanCodecError::InvalidPlan(_))));
}

fn envelope(json: &[u8]) -> String {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(json).unwrap();
    let mut bytes = b"CVP\x01".to_vec();
    bytes.extend(encoder.finish().unwrap());
    URL_SAFE_NO_PAD.encode(bytes)
}
