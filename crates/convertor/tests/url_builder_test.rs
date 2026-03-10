#[allow(unused)]
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
    insta::assert_snapshot!(raw_url.to_string(), @"http://127.0.0.1:8080/api/raw?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&sub_url=iImwC4XKfx_wFPtS_z90X5RlgwMLZx_rEww0IyRDliwS0iO2A75TY4z2P2NQvS26yRldrv67GWkRyABL4uw7VAVN6ebUkKBGjn4Xyp3AYWBcH8j5bKF_54nHIS5r5pHBQz0");

    let profile_url = url_builder.build_profile_url()?;
    insta::assert_snapshot!(profile_url.to_string(), @"http://127.0.0.1:8080/api/profile?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&sub_url=88wW1Vq-k8ffi8eWsfLouIQmLD4YasouA_9tQhI36EeDf-w3wukERaFAZZL7Am_g0SqmcJJ5CMgzLLhkbceOSmq54bP-OiN92hHI-uPmXj3156iy20qtxmfO82pxpdm_lFM");

    let policies = policies();
    let rule_provider_urls = policies
        .iter()
        .map(|policy| url_builder.build_rule_provider_url(policy))
        .map(|url| url.map(|u| u.to_string()))
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    insta::assert_snapshot!(rule_provider_urls, @"
    http://127.0.0.1:8080/api/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D&policy%5Bis_subscription%5D=true&sub_url=-kEIP719vVY4nBzOQkubd0GtBRqTYh1XS1lPTtbLjl70aOeLEKlbMNGzAE_xaSH3je42JVDaL207i-Dw9jZitpPT5QW5xxHruCN3kfTumdTZP8bkm_Xs9_nLafPi0SxCHSk
    http://127.0.0.1:8080/api/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D&policy%5Bis_subscription%5D=false&sub_url=m8wxspjY4LmbI43SrzVd9-g5JK6MqoiuL6JnF9rGbsPh0f4gk5kTQH-XOYOvGkwropUpogfi9K8q4anoV8z2NjLPhlgIaAhNW9aSj2utU3A7x4kKZfl6clGvfpISfkbFhYY
    http://127.0.0.1:8080/api/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D=no-resolve&policy%5Bis_subscription%5D=false&sub_url=breKfdOlzXbfy_6lUz9KhU8qbMXRTv5i64Dhngbi3KD3jorpYCpsJE_VndQOeOyirYx4Ol06-LIGL-tqw8iuXyREfZBNl9SyN2aJE-RUrQlAL1z6BNDNTwB2Okwj2M7cWjE
    http://127.0.0.1:8080/api/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D=force-remote-dns&policy%5Bis_subscription%5D=false&sub_url=n9PCU_KZCmBk-_hJr-8acNo8Eqayz6SfcPffezDVMrn25Bw2DiipLtSikXHTudeblrp4WA_0cmECc3f_9rHerj9mcVZhAf3NC4w1jXcrE6w1Usq8AOPLx3Mz2lZZj9NC0PI
    http://127.0.0.1:8080/api/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D&policy%5Bis_subscription%5D=false&sub_url=16v06FNRPjiZTp0aNjWn-gyaH0DVPdqNeKBd2_cjK5-Ady_q-i21OHV62X0zYUbumgO7hUfZMxIC_zoFdRWHBKUZNSEFqukjenlE9n3alFkv0LupQnDdwZI6mNWFajhhAaM
    http://127.0.0.1:8080/api/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D=no-resolve&policy%5Bis_subscription%5D=false&sub_url=6TsiZSfVo_1i3Nvi5gMhKa5LsQptc1w2tTGA6nIGw2_hmcTqRq2QFFEvVxuV9X-h2y1GHLq3FLaiexK5B8ql5bBBoMVT5_4gSm7KTBBeyqQUky4eoVEz-8N7g0Rg_9SNo7w
    http://127.0.0.1:8080/api/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D=force-remote-dns&policy%5Bis_subscription%5D=false&sub_url=VyzZ34rDtonm0HGBMK05dYPt9KOdcmsLni9JXkV2aHBpCDFngYR5zSGalzpfSi8Hww8R1-s29aR_VWo0mbZCUH7dJzomtATe3IUDoIh3VUkjy3OMHwW7PyszBf-1XST67G4
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
    insta::assert_snapshot!(raw_url.to_string(), @"http://127.0.0.1:8080/api/raw?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&sub_url=SfxJSAferU_ckMID-y4NAqzOwSEai63Q29zkYoBTPL5UWDT7IKdsYBb4ZpxG1kPOTTb2qYeQVUgvBIvLaqawonXysSI1B_7u4RPKmjw-iPrSyPhPoDVxiwi0K04KF82QNVU");

    let profile_url = url_builder.build_profile_url()?;
    insta::assert_snapshot!(profile_url.to_string(), @"http://127.0.0.1:8080/api/profile?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&sub_url=ydgGuWMN4VNtPe__koDVXi2nZPSRnfIGAsDpI1HBcpmaHt1QeeFKlR0ISwVcdsMZf8FQfNHxLM9_yh7tH7ja_MaYjgalH0gg69zKlObGm_Qwk6EvANV2UhDiuEp8i4h89fA");

    let policies = policies();
    let rule_provider_urls = policies
        .iter()
        .map(|policy| url_builder.build_rule_provider_url(policy))
        .map(|url| url.map(|u| u.to_string()))
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    insta::assert_snapshot!(rule_provider_urls, @"
    http://127.0.0.1:8080/api/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D&policy%5Bis_subscription%5D=true&sub_url=ezYMhGpX6Tv4eJs0tkRkH3Dat5ujCVYjOU2v9gPih_NdCt0OQqRoqxejTk84muXA1C1gjvLKfqUjzdJvP7eX4JzJ5QfchW-S5YicO0hmrkSyaqUVGz3gUonV0mZCaVGhEkI
    http://127.0.0.1:8080/api/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D&policy%5Bis_subscription%5D=false&sub_url=g1kwJW_VNvSbKS6WrAGjF8byz5F-8QdOMVVqKVcg58WZT7ehCAoRy_dQ99p7dljHOxHsjs6rYlc3v8DjyyCF3F94AFJ56rZGQxl0yuRO2cS1O1ECynDr6LN1iaV_48aXX0U
    http://127.0.0.1:8080/api/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D=no-resolve&policy%5Bis_subscription%5D=false&sub_url=8GBW8kJV4rJfYqnxtCfmqYLqFPQs1WgkUYZs5GGFpp6aWu5lhHKWmBJ4HiWc7sbvcQmFCO8xgv-GelRy1oqVvIncxuV8Ql-1O-6aU6bzSV84biw5q_BMa4KaZ5hotMHHNBs
    http://127.0.0.1:8080/api/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D=force-remote-dns&policy%5Bis_subscription%5D=false&sub_url=hb5EfgFntmxwBP85w1jLlLQG6AMasTlckKbxQdwgg6PbqCpLsPx2SfVINZyvLaaMDwKKXf-E0e_sTxS6iCQqvi1I-GgSbS6dedlhKNg1irSLlg_iKHKI1Jb3g3F4Kf-yCoU
    http://127.0.0.1:8080/api/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D&policy%5Bis_subscription%5D=false&sub_url=838LWMhjYKCayG-FrHVIn0IXp9yNzahvBxLfbmejxW3LBeeOO8N2RpSJAOdnDVOLfa1SmVJll4e1CtrwZr8SV0gL-8AW7eeWNW0wd5RGH_-EWlopIF0fYBK3ruuF6jYjOnA
    http://127.0.0.1:8080/api/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D=no-resolve&policy%5Bis_subscription%5D=false&sub_url=5tnglcFMVhld81iEKbOEIDyGsfsJ1Bvncd2kjQTxxjx-r7qaHwJIWMlmwEIr0liXg8D9g0AbF3PF5PrXQwe5R2TnTNxBv5mj3AW5BLNqkaLZkEr5Uqoy5tSzUq-TJjLx2dA
    http://127.0.0.1:8080/api/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D=force-remote-dns&policy%5Bis_subscription%5D=false&sub_url=x_x-GEVdgLCq_dOJ0R7FVzZTb79IkcSAVqaSdHawPgCmb0dK48MotE_h16lqEvpvYsUVfdfOPaf2ZMI6dvBZ3Xj4JjcCN8uL1r0oirUxvBKEbqnoaqoqWPew8-w8eKnlozo
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

    let raw_url: ConvUrl = "http://127.0.0.1:8080/raw?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&sub_url=VhmGqJhTbQ6ZEO1Z1J4Z4ODDpeVgBlNJmF55TT9Sqc3EMVmYTeWCIBrClgWdR7uyL5xP6r6SMVBQExU3gXTM3JgEwuw4vQ6ATytNpd1q_4Mq_oOx98i8OHY9kjKOO4Quf9Q".parse()?;
    insta::assert_debug_snapshot!(raw_url);
    let url_builder = UrlBuilder::from_conv_url(encryptor("test_parse_surge_url"), raw_url)?;

    let profile_url: ConvUrl = "http://127.0.0.1:8080/raw?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&sub_url=VhmGqJhTbQ6ZEO1Z1J4Z4ODDpeVgBlNJmF55TT9Sqc3EMVmYTeWCIBrClgWdR7uyL5xP6r6SMVBQExU3gXTM3JgEwuw4vQ6ATytNpd1q_4Mq_oOx98i8OHY9kjKOO4Quf9Q".parse()?;
    insta::assert_debug_snapshot!(profile_url);
    assert_eq!(
        url_builder,
        UrlBuilder::from_conv_url(encryptor("test_parse_surge_url"), profile_url)?
    );

    let rule_provider_urls = r#"
        http://127.0.0.1:8080/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D&policy%5Bis_subscription%5D=true&sub_url=csbww2obE-AM-stg-T4hUtOc4EUYTZgIKVC1aIeEzBDXWADhET6X-12tJ17KPcxQ4Z9q__vFiMoNxZvK7Pjik7sZswftwfziPcrUZnL-eQOk4Xn9hsPVLJo3V2ccgpCADIk
        http://127.0.0.1:8080/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D&policy%5Bis_subscription%5D=false&sub_url=mysnr8S67nT1cVnF3p1x1LSQRY1ddXkRHZoCR_THfSrCGkg8Jy5aJtqYRhfh6rK4HBpTHpMfeFrGdOKmkAtsru_2rzBLajWGdbz0sFOqEbkMLRO74b8ZcyOFEkNcle0VGU4
        http://127.0.0.1:8080/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D=no-resolve&policy%5Bis_subscription%5D=false&sub_url=HMjI5J1AHB5XKuVSRusuS-ikX1h7Z1xYYbNGfXXPCyCv9IC6l6sBrOggikALcXx24-b8wf4iLAhgLk4qWpp4pOr5NnwVduRfhqM_PtX07LWZIWkouUHfAZ9oJjZpL_IWBDA
        http://127.0.0.1:8080/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D=force-remote-dns&policy%5Bis_subscription%5D=false&sub_url=Mbxy7gTDt6ZrdX3jeYdrg8locOh5WBnuPRWkWTSZ4uCg7hILT0E_SyHheMZINNIdAfedc1OHTYxHYyPwOmauWWwI16_anEcIyDIdXyihKKhQ-bsGMIWchKQTmausI9xfRUc
        http://127.0.0.1:8080/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D&policy%5Bis_subscription%5D=false&sub_url=fpBPzIzz7yzfNFq8BR0ugtBeJOlMXqCnd767bVY6teyPVUR8tfZYEq7mYrmV-yBW5PFYGv4gelyJpkiPdYURtXVEMV60EQ_4pQlKM4qBk5PLELZ1cHdSl4pEsGz5E35YNKQ
        http://127.0.0.1:8080/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D=no-resolve&policy%5Bis_subscription%5D=false&sub_url=TdQ4U0w2jhYwO1DPLBbgN0sSfJE27eR-fY602W-VRIIqOf2uUnHRe1F8JYTHEY0Y4KwTYhyeTv3N88LGVZ0xoywjnpRJ3Nv3uIU7eL7X8_Rzs6C-UGMFlnrl89L92MtD_MU
        http://127.0.0.1:8080/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=surge&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D=force-remote-dns&policy%5Bis_subscription%5D=false&sub_url=uH8TV7GTcEk4RXiSvnbQQCetdEVsW0KxF-mJhab1zcUOqz7yvtLAiG2L1i1EyrKj4AcNQ6iYWYoISWDn8ChSYN68fji53jlkko0f05qnMZn0OOGA3GpzZx0tIvJkgGlHwmA
    "#.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| line.parse::<ConvUrl>())
        .collect::<Result<Vec<ConvUrl>, _>>()?;

    for rule_provider_url in rule_provider_urls {
        insta::assert_debug_snapshot!(rule_provider_url);
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

    let raw_url: ConvUrl = "http://127.0.0.1:8080/raw?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&sub_url=NuLmG5c9N0lF_ye5AHSWPudZsHwMQYAU-kMWlp_lXRsAOKbp4Rv7J1_pDt8jI_SG2emWzrgXnRGsxKuvscvNUbIZcUYDkWO5_4aiHmPKQMUimk9wpDNvJkNieI5g6pbC5fY".parse()?;
    insta::assert_debug_snapshot!(raw_url);
    let url_builder = UrlBuilder::from_conv_url(encryptor("test_parse_clash_url"), raw_url)?;

    let profile_url: ConvUrl = "http://127.0.0.1:8080/profile?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&sub_url=u3Psc-uU5vOhHlxgdR9v_CRwpejYvUwgO97nHtygBS--EI95LlRwGS_FpbDtDiEwhNQUC2p3vHyNimDdK-1dLzB91UnEqQDRlyEboCYXQ07nmVPBv7WHM2p872mhrf2g7Ew".parse()?;
    insta::assert_debug_snapshot!(profile_url);
    assert_eq!(
        url_builder,
        UrlBuilder::from_conv_url(encryptor("test_parse_clash_url"), profile_url)?
    );

    let rule_provider_urls = r#"
        http://127.0.0.1:8080/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D&policy%5Bis_subscription%5D=true&sub_url=o1xeolS4y7RGr8oFRVobLyDRK3JvESE8adulCogXmHJKTOhdw2YJrz6ByAik0JomM-yQBuuYoBMlJqr_rfbfCD3dHSHhXtEo7CZUjRSb9PGpDGhkvku_zN7eRGzZWmd92G8
        http://127.0.0.1:8080/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D&policy%5Bis_subscription%5D=false&sub_url=Gb1xIRj0e3SVXiWjl7Ae4Q6LgLmo9OXPaTbcPwdT6R-D3Uc43EvO7q5pKqw2QqfldQfCELdjP34vyX1cmSZ-p3rJQR2yHnB7ZJxE5xpANweqB3Cmz63Fs-d0sTLeax8mMNE
        http://127.0.0.1:8080/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D=no-resolve&policy%5Bis_subscription%5D=false&sub_url=kW8oMPwN60-Lo_46f6Lwz0MlgN25TTW07Jm0Liuibdgkc1Qwch56nI4bx9uzJ5qZdi3hOnPz-DPRJYT4NuidAJPoMYQZhy4iSargk1yLLkFHjN0oAeCi5od3Nl2Z3PRngWA
        http://127.0.0.1:8080/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=BosLife&policy%5Boption%5D=force-remote-dns&policy%5Bis_subscription%5D=false&sub_url=jJQlxJxbOh0wpLsxb-z7FqX_F1wcs5bPmR2n2nvMMfj24WABbZzFgZhnpZg9Tz9t-nlcU-oI5W_BA-RWEIomanpYJbar5_HN_WZwzVYR_IjBh9qfIxKtp5VmvLjunrOG42E
        http://127.0.0.1:8080/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D&policy%5Bis_subscription%5D=false&sub_url=NLJDhoz9FdPwcQKXJxGFZ38gJG5rWIdZaepY2hgT1u9f7T56iTZ8FlIW3QodJLBJINUsXxtWITKDeEOlDe9xJZ_1mD809zCcK-LYPsWrWUMHc6TuFKsuqWpxiDRsWDeifGw
        http://127.0.0.1:8080/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D=no-resolve&policy%5Bis_subscription%5D=false&sub_url=szWnsGoVHyiJurBkAI0-ja46vd9p1_X3LOTUrAzhKL8CgQBJ5TCsLQPFJCRcQKilx_LyWDqdDtwn8OYPOhUTM_dtz7h_jumskVkInO0SSNQWjIcDJmIMWaKIWR3ndgLrMQU
        http://127.0.0.1:8080/rule-provider?server=http%3A%2F%2F127.0.0.1%3A8080%2F&client=clash&interval=86400&strict=true&policy%5Bname%5D=DIRECT&policy%5Boption%5D=force-remote-dns&policy%5Bis_subscription%5D=false&sub_url=V8KJNuVpxDc-ehxzKaP77jZYfzq08jy9qJCZeZeo5St4l6oHtiMO0GB4t1467h59sKgvq-B-r7U-EMuYYTB_50N2A3RIJF2W0IlbKBeYU6fgpSXag9OAwlzaf4S0P3eDrVI
    "#.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| line.parse::<ConvUrl>())
        .collect::<Result<Vec<ConvUrl>, _>>()?;

    for rule_provider_url in rule_provider_urls {
        insta::assert_debug_snapshot!(rule_provider_url);
        assert_eq!(
            url_builder,
            UrlBuilder::from_conv_url(encryptor("test_parse_clash_url"), rule_provider_url)?
        )
    }

    Ok(())
}
