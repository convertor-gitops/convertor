#[allow(unused)]
#[path = "./testkit.rs"]
mod testkit;

use crate::testkit::{encryptor, init_test, policies, subscription_url, url_builder};
use convertor::config::proxy_client::ProxyClient;
use convertor::core::profile::clash_profile::GeoxUrl;
use convertor::url::conv_url::ConvUrl;
use convertor::url::url_builder::UrlBuilder;

#[test]
fn test_build_surge_url() -> color_eyre::Result<()> {
    init_test();
    let url_builder = url_builder(ProxyClient::Surge, "test_build_surge_url")?;
    let raw_url = url_builder.build_original_url()?;
    insta::assert_snapshot!(raw_url.to_string(), @"https://convertor.bppleman.com/subscription?token=bppleman&flag=surge");

    let raw_url = url_builder.build_raw_url()?;
    insta::assert_snapshot!(raw_url.to_string(), @"http://127.0.0.1:8080/subscription/raw?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&sub_url=iImwC4XKfx_wFPtS_z90X5RlgwMLZx_r6elfLMS2vhnML4qrFO08yM3VxM66DctAznbimC4ILKcAqkswpZGv2ZvaXTXsAd7uRGMJHDKfhQrUxpYnemM8hSI79qe5OvxvOYc");

    let profile_url = url_builder.build_profile_url()?;
    insta::assert_snapshot!(profile_url.to_string(), @"http://127.0.0.1:8080/subscription/profile?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&sub_url=88wW1Vq-k8ffi8eWsfLouIQmLD4Yasouz2xSrxD_qrJmAYhPbtzV8Yjg3hAqwBagySFHyg-TEnFtboUwEVhC4tXgz3nnfatGyfpSHOWTnb8HO07KflQrPD867L5vgcIU8-0");

    let policies = policies();
    let rule_provider_urls = policies
        .iter()
        .map(|policy| url_builder.build_rule_provider_url(policy))
        .map(|url| url.map(|u| u.to_string()))
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    insta::assert_snapshot!(rule_provider_urls, @"
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D&policy%5Bis_subscription%5D=true&sub_url=-kEIP719vVY4nBzOQkubd0GtBRqTYh1XAqh8Y-rcZZZfleQwVR9d1N_6qOxcCmxEKheEvR2LP6-XGKlB-rOtildiOLEt619sjAnnrB-ofha9da_n2alGfqjJwFFg_RgYQNA
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D&policy%5Bis_subscription%5D=false&sub_url=m8wxspjY4LmbI43SrzVd9-g5JK6MqoiuXm0if_jTUI6G88nVc5NNPwY09a6RtnXm9qtA3dHbnvn5gIDxoJpeI7I5eZeHrNLcHTMkl1ynrdWnK69KpoMpex6yzVgtmwO_Ddg
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D=no-resolve&policy%5Bis_subscription%5D=false&sub_url=breKfdOlzXbfy_6lUz9KhU8qbMXRTv5iVR6BLzT68iKD7iygfJ9CgOovTJiINx-AHv3gEnlyMaya3FpmPmw8RQeK8qtgy1kMLAPuZLeSO9P3b1Fy8sDd6-p8eW2bO9nvnbo
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D=force-remote-dns&policy%5Bis_subscription%5D=false&sub_url=n9PCU_KZCmBk-_hJr-8acNo8Eqayz6Sfkso_hG9GkxHZngLqZuI_qp_d45ausSvAj7vq293ZLm-1yR2QmGf8eALBtbDwjdjhmxQf_Q7WgKYuVxV6f1ACgjWneOlrahnJyDA
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D&policy%5Bis_subscription%5D=false&sub_url=16v06FNRPjiZTp0aNjWn-gyaH0DVPdqNeeflhrrXsF50qE32xG-RmQlnuU8wBpepIvr-QiJdLhIamSEjGsJuuN-QTUSmJe4htiUQgdcaCzD7LL5I8DUmCQaJgLWGHg40LL8
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D=no-resolve&policy%5Bis_subscription%5D=false&sub_url=6TsiZSfVo_1i3Nvi5gMhKa5LsQptc1w2BbmRIIMhyGg4mtzGaWVRwjYBf3FHgANkp7mmQwzi7FVBSpoAykExmBpd6L2-ua40hUVG2841aO6BygqVFWK6W7gx6fW1Ngwrg6U
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D=force-remote-dns&policy%5Bis_subscription%5D=false&sub_url=VyzZ34rDtonm0HGBMK05dYPt9KOdcmsLYvVDoQEJc09SBD0JaS4MGXMFCx-VoREfqut6DMDnD2irt24fqC0eGcHDwKPnqtgVOIMDAVosDgA2LkLMuxpud7OGSU7lyiBEw0c
    ");

    Ok(())
}

#[test]
fn test_build_clash_url() -> color_eyre::Result<()> {
    init_test();
    let url_builder = url_builder(ProxyClient::Clash, "test_build_clash_url")?;
    let raw_url = url_builder.build_original_url()?;
    insta::assert_snapshot!(raw_url.to_string(), @"https://convertor.bppleman.com/subscription?token=bppleman&flag=clash");

    let raw_url = url_builder.build_raw_url()?;
    insta::assert_snapshot!(raw_url.to_string(), @"http://127.0.0.1:8080/subscription/raw?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&sub_url=SfxJSAferU_ckMID-y4NAqzOwSEai63Qmc_NTOl1bIXJ8Q3oj3UJH4zd6brEe6Eai7YLPY8CuL4Kk9PdpWGx7zRkXCiPtNhaEMxOrmQMg3Ox1kaQ3jYmYoV1Yu91q0SktsQ");

    let profile_url = url_builder.build_profile_url()?;
    insta::assert_snapshot!(profile_url.to_string(), @"http://127.0.0.1:8080/subscription/profile?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&sub_url=ydgGuWMN4VNtPe__koDVXi2nZPSRnfIGRDpMJ8u1AAJ6_cS4TZIknmA4ndSpezqtfwIP7LGq4RJNgv71-h9tPJSFJ1v-oqYdh6jcak1ksoPCy9mZEare_7e6IGVDWIgYoeQ");

    let policies = policies();
    let rule_provider_urls = policies
        .iter()
        .map(|policy| url_builder.build_rule_provider_url(policy))
        .map(|url| url.map(|u| u.to_string()))
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    insta::assert_snapshot!(rule_provider_urls, @"
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D&policy%5Bis_subscription%5D=true&sub_url=ezYMhGpX6Tv4eJs0tkRkH3Dat5ujCVYjw5EqiCAToCL6yoI4qQiREkk7Gphjk-TKoqguuW_mwJlrg5I37VcTzT2FezZkYEx9LTZBoez1QhQsrgI7ARSnVucnuBBsI5bqCZY
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D&policy%5Bis_subscription%5D=false&sub_url=g1kwJW_VNvSbKS6WrAGjF8byz5F-8QdOWl-thu_oo-nLONvXcGU07ywFjmVd5AQa-trPz8QFfROqbRvFZLhBWvtH-cTyi8S_A1kDcBCwiQXZH3KQpOHVfA8R9niaADvw8KY
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D=no-resolve&policy%5Bis_subscription%5D=false&sub_url=8GBW8kJV4rJfYqnxtCfmqYLqFPQs1Wgk15xxomDSgVuUBZGnyRGqx045PuXnFI3X5Pp7PODNAfE3WOTflnHRK2IAs3id7U6FxpW3XOA9267NoWqN-v1naZGR6HlLYK5uwoU
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D=force-remote-dns&policy%5Bis_subscription%5D=false&sub_url=hb5EfgFntmxwBP85w1jLlLQG6AMasTlcXZkLFlfvyAOyFzKfYhaACgLwzqLDMk0Rv7SRHJt4F9HLN4bxhZDeDf_WQpAhS1BKtKCt9RMVVfp4y_qkH3MDf70RR_0EDUC41G0
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D&policy%5Bis_subscription%5D=false&sub_url=838LWMhjYKCayG-FrHVIn0IXp9yNzahvaneyIsQiYQBdPrXx9Tk9wFisJx_NQ-gR4GuxrMygy5iGFYSKkEnFTxKzfiq4bSpMqU_vGpMJ9Ws2j7B0rZIGI8Wz_qQr5-bKvfk
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D=no-resolve&policy%5Bis_subscription%5D=false&sub_url=5tnglcFMVhld81iEKbOEIDyGsfsJ1BvnuoXYXXWOo7qli8w_-IcVIaaK2yKb0E4XGLHn_OiBxCkLrd6TNHtcxHJDHVAlMCGFCF_9FTVYQpXg_QglHabpsSJwa8psdKisV8w
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D=force-remote-dns&policy%5Bis_subscription%5D=false&sub_url=x_x-GEVdgLCq_dOJ0R7FVzZTb79IkcSA2Hzr6dzkMoC5_Hj8lwR988yBURK7oDdPvYCk6qXyVsoujRKKY_0OeUlSiVK3e98oTlJfDC09BElblCwSZlCqzB55qfpMrmbDgxs
    ");

    Ok(())
}

#[test]
fn test_build_download_url() -> color_eyre::Result<()> {
    init_test();
    let url_builder = url_builder(ProxyClient::Clash, "test_build_download_url")?;
    let geox_url = GeoxUrl::default();

    let geoip_download = url_builder.build_download_url(&geox_url.geoip)?;
    insta::assert_snapshot!(geoip_download, @"http://127.0.0.1:8080/download?0%5B0%5D=url&0%5B1%5D=https%3A%2F%2Ftestingcf.jsdelivr.net%2Fgh%2FMetaCubeX%2Fmeta-rules-dat%40release%2Fgeoip.dat");

    let geosite_download = url_builder.build_download_url(&geox_url.geosite)?;
    insta::assert_snapshot!(geosite_download, @"http://127.0.0.1:8080/download?0%5B0%5D=url&0%5B1%5D=https%3A%2F%2Ftestingcf.jsdelivr.net%2Fgh%2FMetaCubeX%2Fmeta-rules-dat%40release%2Fgeosite.dat");

    let mmdb_download = url_builder.build_download_url(&geox_url.mmdb)?;
    insta::assert_snapshot!(mmdb_download, @"http://127.0.0.1:8080/download?0%5B0%5D=url&0%5B1%5D=https%3A%2F%2Ftestingcf.jsdelivr.net%2Fgh%2FMetaCubeX%2Fmeta-rules-dat%40release%2Fcountry.mmdb");

    let asn_download = url_builder.build_download_url(&geox_url.asn)?;
    insta::assert_snapshot!(asn_download, @"http://127.0.0.1:8080/download?0%5B0%5D=url&0%5B1%5D=https%3A%2F%2Fgithub.com%2Fxishang0128%2Fgeoip%2Freleases%2Fdownload%2Flatest%2FGeoLite2-ASN.mmdb");

    Ok(())
}

#[test]
fn test_parse_surge_url() -> color_eyre::Result<()> {
    init_test();

    let mut sub_url = subscription_url()?;
    sub_url.query_pairs_mut().append_pair("flag", "surge");
    let original_url: ConvUrl = sub_url.try_into()?;
    insta::assert_debug_snapshot!(original_url);
    assert!(UrlBuilder::from_conv_url(encryptor("test_parse_surge_url"), original_url).is_err());

    let raw_url: ConvUrl = "http://127.0.0.1:8080/subscription/raw?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&sub_url=iImwC4XKfx_wFPtS_z90X5RlgwMLZx_r6elfLMS2vhnML4qrFO08yM3VxM66DctAznbimC4ILKcAqkswpZGv2ZvaXTXsAd7uRGMJHDKfhQrUxpYnemM8hSI79qe5OvxvOYc".parse()?;
    insta::assert_debug_snapshot!(raw_url);
    let url_builder = UrlBuilder::from_conv_url(encryptor("test_parse_surge_url"), raw_url)?;

    let profile_url: ConvUrl = "http://127.0.0.1:8080/subscription/profile?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&sub_url=88wW1Vq-k8ffi8eWsfLouIQmLD4Yasouz2xSrxD_qrJmAYhPbtzV8Yjg3hAqwBagySFHyg-TEnFtboUwEVhC4tXgz3nnfatGyfpSHOWTnb8HO07KflQrPD867L5vgcIU8-0".parse()?;
    insta::assert_debug_snapshot!(profile_url);
    assert_eq!(
        url_builder,
        UrlBuilder::from_conv_url(encryptor("test_parse_surge_url"), profile_url)?
    );

    let rule_provider_urls = r#"
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D&policy%5Bis_subscription%5D=true&sub_url=-kEIP719vVY4nBzOQkubd0GtBRqTYh1XAqh8Y-rcZZZfleQwVR9d1N_6qOxcCmxEKheEvR2LP6-XGKlB-rOtildiOLEt619sjAnnrB-ofha9da_n2alGfqjJwFFg_RgYQNA
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D&policy%5Bis_subscription%5D=false&sub_url=m8wxspjY4LmbI43SrzVd9-g5JK6MqoiuXm0if_jTUI6G88nVc5NNPwY09a6RtnXm9qtA3dHbnvn5gIDxoJpeI7I5eZeHrNLcHTMkl1ynrdWnK69KpoMpex6yzVgtmwO_Ddg
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D=no-resolve&policy%5Bis_subscription%5D=false&sub_url=breKfdOlzXbfy_6lUz9KhU8qbMXRTv5iVR6BLzT68iKD7iygfJ9CgOovTJiINx-AHv3gEnlyMaya3FpmPmw8RQeK8qtgy1kMLAPuZLeSO9P3b1Fy8sDd6-p8eW2bO9nvnbo
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D=force-remote-dns&policy%5Bis_subscription%5D=false&sub_url=n9PCU_KZCmBk-_hJr-8acNo8Eqayz6Sfkso_hG9GkxHZngLqZuI_qp_d45ausSvAj7vq293ZLm-1yR2QmGf8eALBtbDwjdjhmxQf_Q7WgKYuVxV6f1ACgjWneOlrahnJyDA
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D&policy%5Bis_subscription%5D=false&sub_url=16v06FNRPjiZTp0aNjWn-gyaH0DVPdqNeeflhrrXsF50qE32xG-RmQlnuU8wBpepIvr-QiJdLhIamSEjGsJuuN-QTUSmJe4htiUQgdcaCzD7LL5I8DUmCQaJgLWGHg40LL8
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D=no-resolve&policy%5Bis_subscription%5D=false&sub_url=6TsiZSfVo_1i3Nvi5gMhKa5LsQptc1w2BbmRIIMhyGg4mtzGaWVRwjYBf3FHgANkp7mmQwzi7FVBSpoAykExmBpd6L2-ua40hUVG2841aO6BygqVFWK6W7gx6fW1Ngwrg6U
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D=force-remote-dns&policy%5Bis_subscription%5D=false&sub_url=VyzZ34rDtonm0HGBMK05dYPt9KOdcmsLYvVDoQEJc09SBD0JaS4MGXMFCx-VoREfqut6DMDnD2irt24fqC0eGcHDwKPnqtgVOIMDAVosDgA2LkLMuxpud7OGSU7lyiBEw0c
    "#.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| line.parse::<ConvUrl>())
        .collect::<Result<Vec<ConvUrl>, _>>()?;

    insta::assert_debug_snapshot!(rule_provider_urls);

    for rule_provider_url in rule_provider_urls {
        assert_eq!(
            url_builder,
            UrlBuilder::from_conv_url(encryptor("test_parse_surge_url"), rule_provider_url)?
        )
    }

    Ok(())
}

#[test]
fn test_parse_clash_url() -> color_eyre::Result<()> {
    init_test();

    let mut sub_url = subscription_url()?;
    sub_url.query_pairs_mut().append_pair("flag", "clash");
    let original_url: ConvUrl = sub_url.try_into()?;
    insta::assert_debug_snapshot!(original_url);
    assert!(UrlBuilder::from_conv_url(encryptor("test_parse_clash_url"), original_url).is_err());

    let raw_url: ConvUrl = "http://127.0.0.1:8080/subscription/raw?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&sub_url=SfxJSAferU_ckMID-y4NAqzOwSEai63Qmc_NTOl1bIXJ8Q3oj3UJH4zd6brEe6Eai7YLPY8CuL4Kk9PdpWGx7zRkXCiPtNhaEMxOrmQMg3Ox1kaQ3jYmYoV1Yu91q0SktsQ".parse()?;
    insta::assert_debug_snapshot!(raw_url);
    let url_builder = UrlBuilder::from_conv_url(encryptor("test_parse_clash_url"), raw_url)?;

    let profile_url: ConvUrl = "http://127.0.0.1:8080/subscription/profile?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&sub_url=ydgGuWMN4VNtPe__koDVXi2nZPSRnfIGRDpMJ8u1AAJ6_cS4TZIknmA4ndSpezqtfwIP7LGq4RJNgv71-h9tPJSFJ1v-oqYdh6jcak1ksoPCy9mZEare_7e6IGVDWIgYoeQ".parse()?;
    insta::assert_debug_snapshot!(profile_url);
    assert_eq!(
        url_builder,
        UrlBuilder::from_conv_url(encryptor("test_parse_clash_url"), profile_url)?
    );

    let rule_provider_urls = r#"
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D&policy%5Bis_subscription%5D=true&sub_url=ezYMhGpX6Tv4eJs0tkRkH3Dat5ujCVYjw5EqiCAToCL6yoI4qQiREkk7Gphjk-TKoqguuW_mwJlrg5I37VcTzT2FezZkYEx9LTZBoez1QhQsrgI7ARSnVucnuBBsI5bqCZY
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D&policy%5Bis_subscription%5D=false&sub_url=g1kwJW_VNvSbKS6WrAGjF8byz5F-8QdOWl-thu_oo-nLONvXcGU07ywFjmVd5AQa-trPz8QFfROqbRvFZLhBWvtH-cTyi8S_A1kDcBCwiQXZH3KQpOHVfA8R9niaADvw8KY
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D=no-resolve&policy%5Bis_subscription%5D=false&sub_url=8GBW8kJV4rJfYqnxtCfmqYLqFPQs1Wgk15xxomDSgVuUBZGnyRGqx045PuXnFI3X5Pp7PODNAfE3WOTflnHRK2IAs3id7U6FxpW3XOA9267NoWqN-v1naZGR6HlLYK5uwoU
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D=force-remote-dns&policy%5Bis_subscription%5D=false&sub_url=hb5EfgFntmxwBP85w1jLlLQG6AMasTlcXZkLFlfvyAOyFzKfYhaACgLwzqLDMk0Rv7SRHJt4F9HLN4bxhZDeDf_WQpAhS1BKtKCt9RMVVfp4y_qkH3MDf70RR_0EDUC41G0
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D&policy%5Bis_subscription%5D=false&sub_url=838LWMhjYKCayG-FrHVIn0IXp9yNzahvaneyIsQiYQBdPrXx9Tk9wFisJx_NQ-gR4GuxrMygy5iGFYSKkEnFTxKzfiq4bSpMqU_vGpMJ9Ws2j7B0rZIGI8Wz_qQr5-bKvfk
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D=no-resolve&policy%5Bis_subscription%5D=false&sub_url=5tnglcFMVhld81iEKbOEIDyGsfsJ1BvnuoXYXXWOo7qli8w_-IcVIaaK2yKb0E4XGLHn_OiBxCkLrd6TNHtcxHJDHVAlMCGFCF_9FTVYQpXg_QglHabpsSJwa8psdKisV8w
        http://127.0.0.1:8080/subscription/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D=force-remote-dns&policy%5Bis_subscription%5D=false&sub_url=x_x-GEVdgLCq_dOJ0R7FVzZTb79IkcSA2Hzr6dzkMoC5_Hj8lwR988yBURK7oDdPvYCk6qXyVsoujRKKY_0OeUlSiVK3e98oTlJfDC09BElblCwSZlCqzB55qfpMrmbDgxs
    "#.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| line.parse::<ConvUrl>())
        .collect::<Result<Vec<ConvUrl>, _>>()?;

    insta::assert_debug_snapshot!(rule_provider_urls);
    for rule_provider_url in rule_provider_urls {
        assert_eq!(
            url_builder,
            UrlBuilder::from_conv_url(encryptor("test_parse_clash_url"), rule_provider_url)?
        )
    }

    Ok(())
}
