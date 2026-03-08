use crate::core::profile::clash_profile::ClashProfile;
use crate::core::profile::rule::Rule;
use crate::error::ParseError;
use serde_yml::{Value, from_str, from_value};
use tracing::instrument;

type Result<T> = core::result::Result<T, ParseError>;

pub struct ClashParser;

impl ClashParser {
    #[instrument(skip_all)]
    pub fn parse(raw_profile: impl AsRef<str>) -> Result<ClashProfile> {
        Ok(from_str(raw_profile.as_ref())?)
    }

    #[instrument(skip_all)]
    pub fn parse_rules(section: impl AsRef<str>) -> Result<Vec<Rule>> {
        let value: Value = from_str(section.as_ref())?;
        let rules = match value {
            Value::Sequence(rules) => rules.into_iter().map(|r| Ok(from_value(r)?)).collect::<Result<Vec<Rule>>>(),
            Value::Mapping(mut rules) => {
                if rules.contains_key("rules") {
                    rules.remove("rules").map(|v| Ok(from_value(v)?)).ok_or(ParseError::Rule {
                        line: 0,
                        reason: "rules 无法反序列化为 Rule 序列".to_string(),
                    })?
                } else if rules.contains_key("payload") {
                    rules
                        .remove("payload")
                        .map(|v| Ok(serde_yml::from_value(v)?))
                        .ok_or(ParseError::Rule {
                            line: 0,
                            reason: "payload 无法反序列化为 Rule 序列".to_string(),
                        })?
                } else {
                    Err(ParseError::Rule {
                        line: 0,
                        reason: "没有找到 rules 或 payload".to_string(),
                    })
                }
            }
            _ => Err(ParseError::Rule {
                line: 0,
                reason: "反序列化规则应当传入一个规则序列或以`rules:`/`payload:`开头的映射".to_string(),
            }),
        }?;
        Ok(rules)
    }
}
