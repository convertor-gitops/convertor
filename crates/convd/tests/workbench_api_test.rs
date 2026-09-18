#[allow(dead_code)]
#[path = "./testkit.rs"]
mod testkit;

use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode},
};
use convd::server::{app_state::AppState, router};
use convertor::{
    config::proxy_client::ProxyClient,
    core::plan::{
        Builtin, CustomGroup, EmptyGroupPolicy, GroupId, GroupStrategy, NodePredicate, NodeSelection, Output, Plan, Predicate, Source,
        SourceId, SourceInput, StringMatch, Target, encode_plan,
    },
    subscription::SourceProfile,
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

fn plan() -> Plan {
    let content = "[General]\nloglevel = notify\n\n[Proxy]\nHK = trojan, example.com, 443, password=secret\n\n[Proxy Group]\n\n[Rule]\nFINAL,DIRECT\n";
    Plan {
        version: 1,
        client: ProxyClient::Surge,
        sources: vec![Source {
            id: SourceId(1),
            name: "inline".into(),
            input: SourceInput::Inline { content: content.into() },
            annotations: vec![],
            node_filter: None,
        }],
        grouping_policies: vec![],
        groups: vec![],
        rules: vec![],
        output: Output {
            roots: vec![],
            extra_nodes: vec![NodeSelection {
                source: SourceId(1),
                predicate: Predicate::<NodePredicate>::All(vec![]),
            }],
            fallback: Target::Builtin(Builtin::Direct),
            settings_source: SourceId(1),
        },
    }
}

async fn post(router: &axum::Router, path: &str, body: Value) -> color_eyre::Result<Value> {
    let request = Request::builder()
        .uri(path)
        .method(Method::POST)
        .header("host", "workbench.test")
        .header("x-forwarded-proto", "https")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body)?))?;
    let response = router.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);
    Ok(serde_json::from_slice(&response.into_body().collect().await?.to_bytes())?)
}

#[tokio::test]
async fn workbench_flow_loads_evaluates_builds_decodes_and_executes() -> color_eyre::Result<()> {
    let router = router::router(AppState::new(testkit::test_config("https://provider.example/subscription")?, None));
    let plan = plan();
    let load = post(
        &router,
        "/api/load-source",
        json!({
            "source_id": 1,
            "client": "surge",
            "input": plan.sources[0].input,
            "cache": "use"
        }),
    )
    .await?;
    assert_eq!(load["status"]["status"], "OK");
    assert_eq!(load["data"]["input_nodes"][0]["identity"], "s1/n0");
    assert_eq!(load["data"]["input_nodes"][0]["source"], 1);
    assert_eq!(load["data"]["input_nodes"][0]["proxy"]["name"], "HK");
    assert_eq!(load["data"]["input_nodes"][0]["origins"][0]["kind"], "direct");
    assert!(
        load["data"]["input_nodes"][0]["region"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
    assert_eq!(load["data"]["input_diagnostics"], json!([]));
    let source_profile: SourceProfile = serde_json::from_value(load["data"].clone())?;

    let evaluation = post(
        &router,
        "/api/evaluate-plan",
        json!({"plan": plan, "source_profiles": [source_profile.clone()]}),
    )
    .await?;
    assert_eq!(evaluation["status"]["status"], "OK");
    assert_eq!(evaluation["data"]["report"]["nodes"].as_array().unwrap().len(), 1);
    assert_eq!(
        load["data"]["input_nodes"][0]["identity"],
        evaluation["data"]["report"]["nodes"][0]["identity"]
    );
    let preview = evaluation["data"]["rendered_content"].as_str().unwrap();
    assert!(preview.contains("HK=trojan"), "{preview}");

    let mut stale = source_profile.clone();
    stale.fingerprint = "tampered".into();
    let rejected = post(&router, "/api/evaluate-plan", json!({"plan": plan, "source_profiles": [stale]})).await?;
    assert_eq!(rejected["status"]["status"], "PLAN_EVALUATION_ERROR");
    assert!(rejected.get("data").is_none());
    assert!(rejected["messages"].as_array().unwrap().iter().any(|message| {
        message
            .as_str()
            .is_some_and(|message| message.contains("invalid_source_profile_fingerprint at source/1"))
    }));

    let mut invalid_plan = plan.clone();
    invalid_plan.output.roots = vec![GroupId(404)];
    let invalid_preview = post(
        &router,
        "/api/evaluate-plan",
        json!({"plan": invalid_plan, "source_profiles": [source_profile]}),
    )
    .await?;
    assert_eq!(invalid_preview["status"]["status"], "OK");
    assert!(invalid_preview["data"]["rendered_content"].is_null());
    assert_eq!(invalid_preview["data"]["report"]["diagnostics"][0]["code"], "invalid_plan");

    let built = post(&router, "/api/build-url", json!({"plan": plan})).await?;
    assert_eq!(built["status"]["status"], "OK");
    let subscription_url = built["data"]["url"].as_str().unwrap();
    assert!(subscription_url.starts_with("https://workbench.test/subscription/profile?plan="));

    let decoded = post(&router, "/api/decode-plan", json!({"url": subscription_url})).await?;
    assert_eq!(decoded["status"]["status"], "OK");
    assert_eq!(decoded["data"]["plan"]["version"], 1);

    let url = url::Url::parse(subscription_url)?;
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri(url[url::Position::BeforePath..].to_owned())
                .method(Method::GET)
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let rendered = String::from_utf8(response.into_body().collect().await?.to_bytes().to_vec())?;
    assert!(rendered.contains("HK=trojan"));
    assert!(rendered.contains("FINAL,DIRECT"));
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/subscription/profile?plan=invalid!")
                .method(Method::GET)
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let mut blocked = plan.clone();
    blocked.output.extra_nodes.clear();
    blocked.groups.push(CustomGroup {
        id: GroupId(1),
        name: "empty".into(),
        strategy: GroupStrategy::Select,
        member_selectors: vec![convertor::core::plan::MemberSelector::Nodes(NodeSelection {
            source: SourceId(1),
            predicate: Predicate::Atom(NodePredicate::Name(StringMatch::Equals("missing".into()))),
        })],
        on_empty: EmptyGroupPolicy::Error,
    });
    blocked.output.roots = vec![GroupId(1)];
    blocked.output.fallback = Target::Group(GroupId(1));
    let encoded = encode_plan(&blocked)?.value;
    let response = router
        .oneshot(
            Request::builder()
                .uri(format!("/subscription/profile?plan={encoded}"))
                .method(Method::GET)
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    Ok(())
}
