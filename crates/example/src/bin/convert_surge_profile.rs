use convertor::common::once::{init_backtrace, init_base_dir};
use convertor::config::Config;
use convertor::config::proxy_client::ProxyClient;
use convertor::core::profile::ProfileTrait;
use convertor::core::profile::surge_profile::SurgeProfile;
use convertor::core::renderer::Renderer;
use convertor::core::renderer::surge_renderer::SurgeRenderer;
use convertor::provider::SubsProvider;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> color_eyre::Result<()> {
    let base_dir = init_base_dir();
    init_backtrace(|| {
        if let Err(e) = color_eyre::install() {
            eprintln!("Failed to install color_eyre: {e}");
        }
    });

    // 搜索可用配置文件
    let config: Config = Config::search(&base_dir, Option::<&str>::None)?;
    // 创建订阅供应商实例
    let provider = SubsProvider::new(None, config.redis.as_ref().map(|r| r.prefix.as_str()));

    // 获取原始订阅配置文件内容: 来源于 BosLife 机场;适用于 Surge
    let sub_url = config.subscription.sub_url.clone();
    let raw_sub_content = provider.get_raw_profile(sub_url, [("User-Agent", "Surge Mac/8310")].into()).await?;
    // 解析原始配置文件内容为 SurgeProfile 对象
    let mut profile = SurgeProfile::parse(raw_sub_content)?;
    // 创建 UrlBuilder 对象, 该 UrlBuilder 可用于创建适用于 Surge 的且使用 BosLife 订阅的 URL
    let url_builder = config.create_url_builder(ProxyClient::Surge)?;
    // 转换 SurgeProfile 对象
    // 传入 UrlBuilder 对象有两个作用
    // - 用于生成 Surge 配置的托管链接
    // - 用于生成 Surge 规则集的托管链接
    // 二者均会指向 convertor 所在服务器
    profile.convert(&url_builder)?;

    // 使用渲染器将 SurgeProfile 对象转换为字符串格式
    let converted = SurgeRenderer::render_profile(&profile)?;
    println!("{converted}");

    Ok(())
}
