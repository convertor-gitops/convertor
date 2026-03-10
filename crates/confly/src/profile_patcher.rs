use crate::config::ClientConfig;
use crate::file_provider::FileProvider;
use convertor::core::profile::clash_profile::ClashProfile;
use convertor::core::profile::policy::Policy;
use convertor::core::profile::rule::Rule;
use convertor::core::profile::surge_header::SurgeHeader;
use convertor::core::renderer::Renderer;
use convertor::core::renderer::surge_renderer::{SURGE_RULE_PROVIDER_COMMENT_END, SURGE_RULE_PROVIDER_COMMENT_START, SurgeRenderer};
use convertor::url::conv_url::UrlType;
use convertor::url::url_builder::UrlBuilder;
use std::borrow::Cow;

impl ClientConfig {
    pub fn update_surge_config<'a>(
        &self,
        file_provider: &FileProvider,
        url_builder: &UrlBuilder,
        policies: impl IntoIterator<Item = &'a Policy>,
    ) -> color_eyre::Result<()> {
        // 更新主订阅配置，即由 convertor 生成的订阅配置
        let main_profile = Self::update_surge_conf(
            file_provider.read(self.main_profile_path())?,
            url_builder.build_surge_header(UrlType::Profile)?,
        )?;
        file_provider.write(self.main_profile_path(), main_profile)?;

        if let Some(path) = self.raw_path() {
            let raw = Self::update_surge_conf(file_provider.read(&path)?, url_builder.build_surge_header(UrlType::Original)?)?;
            file_provider.write(path, raw)?;
        }

        // 更新转发原始订阅配置，即由 convertor 生成的原始订阅配置
        if let Some(path) = self.raw_profile_path() {
            let raw_profile = Self::update_surge_conf(file_provider.read(&path)?, url_builder.build_surge_header(UrlType::Raw)?)?;
            file_provider.write(path, raw_profile)?;
        }

        // 更新 rules.dconf 中的 RULE-SET 规则，规则提供者将从 policies 中生成 URL
        if let Some(path) = self.rules_path() {
            let rules = Self::update_surge_rule_providers(file_provider.read(&path)?, url_builder, policies)?;
            file_provider.write(path, rules)?;
        }

        Ok(())
    }

    fn update_surge_conf(mut content: String, header: SurgeHeader) -> color_eyre::Result<String> {
        let mut lines = content.lines().map(Cow::Borrowed).collect::<Vec<_>>();
        if lines.is_empty() {
            Ok(header.to_string())
        } else {
            lines[0] = Cow::Owned(header.to_string());
            content = lines.join("\n");
            Ok(content)
        }
    }

    fn update_surge_rule_providers<'a>(
        content: String,
        url_builder: &UrlBuilder,
        policies: impl IntoIterator<Item = &'a Policy>,
    ) -> color_eyre::Result<String> {
        let mut lines = content.lines().map(Cow::Borrowed).collect::<Vec<_>>();

        let range_of_rule_providers = lines.iter().enumerate().fold(0..=0, |acc, (no, line)| {
            let mut start = *acc.start();
            let mut end = *acc.end();
            if line == SURGE_RULE_PROVIDER_COMMENT_START {
                start = no;
            } else if line == SURGE_RULE_PROVIDER_COMMENT_END {
                end = no;
            }
            start..=end
        });

        let provider_rules = policies
            .into_iter()
            .map(|policy| {
                let name = SurgeRenderer::render_provider_name_for_policy(policy);
                let url = url_builder.build_rule_provider_url(policy)?;
                Ok(Rule::surge_rule_set(policy, name, url))
            })
            .collect::<color_eyre::Result<Vec<_>>>()?;
        let mut output = provider_rules
            .iter()
            .map(SurgeRenderer::render_rule)
            .map(|l| Ok(l.map(Cow::Owned)?))
            .collect::<color_eyre::Result<Vec<_>>>()?;
        output.insert(0, Cow::Borrowed(SURGE_RULE_PROVIDER_COMMENT_START));
        output.push(Cow::Borrowed(SURGE_RULE_PROVIDER_COMMENT_END));
        lines.splice(range_of_rule_providers, output);
        Ok(lines.join("\n"))
    }

    pub fn update_clash_config(
        &self,
        file_provider: &FileProvider,
        url_builder: &UrlBuilder,
        raw_profile: ClashProfile,
        secret: impl AsRef<str>,
    ) -> color_eyre::Result<()> {
        todo!()
        // let mut template = ClashProfile::template()?;
        // template.patch(raw_profile)?;
        // template.convert(url_builder)?;
        // template.secret = Some(secret.as_ref().to_string());
        // let main_profile = ClashRenderer::render_profile(&template)?;
        // file_provider.write(self.main_profile_path(), main_profile)?;
        // Ok(())
    }
}
