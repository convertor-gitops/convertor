//! `cargo run -p convertor --example plan -- surge|clash [--plan]`
use convertor::config::proxy_client::ProxyClient;
use convertor::core::evaluator::{EvaluationSource, ResolvedDependencies, evaluate};
use convertor::core::plan::*;
use convertor::core::profile::Profile;
use convertor::core::{Parse, Render};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let client = if args.first().is_some_and(|v| v == "clash") {
        ProxyClient::Clash
    } else {
        ProxyClient::Surge
    };
    let contents = match client {
        ProxyClient::Surge => vec![
            "[General]\nloglevel=notify\n[Proxy]\n香港 01=trojan,hk.example.com,443,password=example\n美国 07=trojan,us.example.com,443,password=example\n[Proxy Group]\n[Rule]\nFINAL,DIRECT\n",
            "[General]\n[Proxy]\n美国 01=trojan,us.example.org,443,password=example\n加拿大 01=trojan,ca.example.org,443,password=example\n[Proxy Group]\n[Rule]\nFINAL,DIRECT\n",
        ],
        ProxyClient::Clash => vec![
            "proxies:\n  - {name: 香港 01, type: trojan, server: hk.example.com, port: 443, password: example}\n  - {name: 美国 07, type: trojan, server: us.example.com, port: 443, password: example}\n",
            "proxies:\n  - {name: 美国 01, type: trojan, server: us.example.org, port: 443, password: example}\n  - {name: 加拿大 01, type: trojan, server: ca.example.org, port: 443, password: example}\n",
        ],
    };
    let sources = contents
        .iter()
        .enumerate()
        .map(|(i, content)| Source {
            id: SourceId(i as u32 + 1),
            name: format!("source-{}", if i == 0 { "a" } else { "b" }),
            input: SourceInput::Inline {
                content: content.to_string(),
            },
            annotations: vec![],
            node_filter: None,
        })
        .collect();
    let plan = Plan {
        version: 1,
        client,
        sources,
        grouping_policies: vec![GroupingPolicy {
            id: GroupingPolicyId(1),
            group_by: vec![NodeDimension::Region, NodeDimension::Source],
            strategy: GroupStrategy::Select,
        }],
        groups: vec![CustomGroup {
            id: GroupId(1),
            name: "all".into(),
            strategy: GroupStrategy::Select,
            member_selectors: vec![MemberSelector::BaseGroups(BaseGroupSelection {
                policy: GroupingPolicyId(1),
                scope: GroupScope::Roots,
                predicate: Predicate::All(vec![]),
            })],
            on_empty: EmptyGroupPolicy::Error,
        }],
        rules: vec![],
        output: Output {
            roots: vec![GroupId(1)],
            extra_nodes: vec![],
            fallback: Target::Group(GroupId(1)),
            settings_source: SourceId(1),
        },
    };
    if args.iter().any(|v| v == "--plan") {
        println!("{}", serde_json::to_string_pretty(&plan)?);
        return Ok(());
    }
    let sources = contents
        .iter()
        .enumerate()
        .map(|(i, content)| {
            let profile = Profile::parse(content, client)?;
            Ok(EvaluationSource {
                source_id: SourceId(i as u32 + 1),
                client,
                profile,
            })
        })
        .collect::<Result<Vec<_>, convertor::error::ParseError>>()?;
    let result = evaluate(&plan, &sources, &ResolvedDependencies::default()).map_err(|e| format!("{:?}", e.diagnostics))?;
    let mut content = String::new();
    result.profile.render(&mut content, client)?;
    print!("{content}");
    Ok(())
}
