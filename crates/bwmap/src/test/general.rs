use crate::test::get_all_test_chks;
use crate::test::get_chk;
use crate::ParsedChk;
use futures::pin_mut;
use futures::TryStreamExt;

#[tokio::test]
async fn test_parse_merged_chunks() {
    let stream = get_all_test_chks();

    pin_mut!(stream);

    while let Some(chk) = stream.try_next().await.unwrap() {
        let parsed_chk = ParsedChk::from_bytes(chk.as_slice());

        assert!(
            parsed_chk.vcod.is_ok(),
            "chk_data.len(): {}, {:?}",
            chk.len(),
            parsed_chk
        );
    }
}

#[tokio::test]
async fn test_specific_map_0c0c_bound_protected_by_acmp_version_1_dot_74() {
    let chk = get_chk("541bb96fdd38d14a6dc2cd877fa80d480e2e52ccd96e76e17e276deac4d23f52")
        .await
        .unwrap();

    let parsed_chk = ParsedChk::from_bytes(chk.as_slice());

    assert!(parsed_chk.vcod.is_ok());
}

#[tokio::test]
async fn test_specific_map_sniper_seed_protected_by_smc_version_2_dot_9() {
    let chk = get_chk("e1020e7169d92ffba63b44fa52f52ce8dd4281aaa0a27767518280ceb7b13d50")
        .await
        .unwrap();

    let parsed_chk = ParsedChk::from_bytes(chk.as_slice());

    assert!(parsed_chk.vcod.is_ok());
}

#[tokio::test]
async fn test_specific_map_protected_by_smlp_version_2_dot_5_dot_00() {
    let chk = get_chk("bb2d1dd37853f60d35b372b4280e80956407aa1be21c6fd89275a18a0edc15f5")
        .await
        .unwrap();

    let parsed_chk = ParsedChk::from_bytes(chk.as_slice());

    assert!(parsed_chk.vcod.is_ok());
}

#[tokio::test]
async fn test_specific_map_protected_by_unknown_protector() {
    // Unknown protector with a bunch of MPQ hacks?
    let chk = get_chk("1d901afa80c7cf8647a35c1d4cc2f5baa8a81928efdc860e169e50f5e77d7f8d")
        .await
        .unwrap();

    let parsed_chk = ParsedChk::from_bytes(chk.as_slice());

    assert!(parsed_chk.vcod.is_ok());
}

#[tokio::test]
async fn test_get_string_on_all_maps() {
    let stream = get_all_test_chks();

    pin_mut!(stream);

    while let Some(chk) = stream.try_next().await.unwrap() {
        let parsed_chk = ParsedChk::from_bytes(chk.as_slice());
        let string_refs = parsed_chk.get_all_string_references().unwrap();

        for string_ref in string_refs {
            parsed_chk.get_string(string_ref as usize).unwrap();
        }
    }
}

#[tokio::test]
async fn test_constrain_encoding_detection_algorithm() {
    let f = |s: String| async move {
        let chk = get_chk(s.as_str()).await.unwrap();
        let parsed_chk = ParsedChk::from_bytes(chk.as_slice());

        let sprp_scenario_index = if let Ok(x) = &parsed_chk.sprp {
            *x.scenario_name_string_number
        } else {
            unreachable!();
        };

        parsed_chk
            .get_string(sprp_scenario_index as usize)
            .unwrap()
            .to_owned()
    };

    #[rustfmt::skip]
    let test_vectors = [
        ("427b6ebfbdc97bcfe7e2cadc8f98799cee01cd812bd69a106616e27335edc3f1", "폭피[뿌요뿌요]"),
        ("96cd0ccfd0cd89b9fe4e9bb68e7548645e74e40a92ed40637c11f185fff49815", "JØNÎ$  ßøûñÐ(beta)"),
        ("b69c56d44a60f41bf08b6c3d8dcbf30bc1d1c0bbbe806be8a98866cd5694607e", "\u{0013}\u{0002}Ðúst BóüÑÐ\u{0012}\u{0006}Dust BouND      ."),
        ("e0b364765d1ae37159f81421d914dd9941c858a1c4173bd5cc7c4cf0aaf9ce32", "\u{4}Poo\u{6}p \u{3}Boun\u{6}d"),
        ("fd05963c147bb4ba687fc276b45b5f17590f0be8ea7d65fbfd649613532c8f0e", "\u{3}C\u{4}r\u{6}escent \u{7}B\u{6}ound"),
        ("bd7b3e491caf2d239a1dfc6aa0f9a74bd8064393c8467411ff4e9aaac10449c8", "\u{4}도라에몽\u{4} 의\u{4} \u{4}S\u{5}unken "),
        ("997663060cf54f7795f3f826c1babbff0284a9f70072865b5dfc4b1d4a14de6d", "\u{1}마린키우기 \u{7}E\u{6}cstasy \u{3}EVF"),
        ("1d901afa80c7cf8647a35c1d4cc2f5baa8a81928efdc860e169e50f5e77d7f8d", "갓 타워디펜스4VZ015겨울"),
        ("79974153879019b797282881b4b7e9b8f0a62050cea3897d27d27195add07bb9", "\u{2}Can \u{1}You \u{2}Stop \u{1}1 \u{4}Unit? \u{3}§tack"),
        ("d3e7310b02fc5f296299b6dd6c22f9850db879407f39c8244cb794a385505fa3", "\u{3}Marine Special Forces \u{7}Re"),
    ];

    for (a, b) in test_vectors.into_iter() {
        assert_eq!(b, f(a.to_owned()).await);
    }
}

#[tokio::test]
async fn test_constrain_encoding_detection_algorithm2() {
    let f = |s: String, index: usize| async move {
        let chk = get_chk(s.as_str()).await.unwrap();
        let parsed_chk = ParsedChk::from_bytes(chk.as_slice());
        parsed_chk.get_string(index).unwrap()
    };

    #[rustfmt::skip]
    let test_vectors = [
        ("d3e7310b02fc5f296299b6dd6c22f9850db879407f39c8244cb794a385505fa3", 1,  "\u{3}Marine Special Forces \u{7}Re"),
        ("d3e7310b02fc5f296299b6dd6c22f9850db879407f39c8244cb794a385505fa3", 2,  "[오리지널 \"Marine Special Forces\" 헌정맵]\r\n오리지널 마린키우기를 리메이크 했습니다.\r\n기존에 있던 마린키우기를 수정하는 것이\r\n아니라 새롭게 제작하였습니다.\r\n\r\n수정일 : 19.07.02\r\n버전 : 정식 1.62\r\n제작 : 리메이커"),
        ("0858b5e157344d80ee0d250c8189c41d52d9f91481838ee88c2d948fff0006d2", 1,  "\u{3}Marine Special Forces \u{7}Re"),
        ("0858b5e157344d80ee0d250c8189c41d52d9f91481838ee88c2d948fff0006d2", 2,  "[오리지널 \"Marine Special Forces\" 헌정맵]\r\n오리지널 마린키우기를 리메이크 했습니다.\r\n기존에 있던 마린키우기를 수정하는 것이\r\n아니라 새롭게 제작하였습니다.\r\n\r\n수정일 : 19.10.06\r\n버전 : 정식 1.31 [확장판]\r\n제작 : 리메이커"),
        ("79974153879019b797282881b4b7e9b8f0a62050cea3897d27d27195add07bb9", 23, "쓰레기통"),
        ("79974153879019b797282881b4b7e9b8f0a62050cea3897d27d27195add07bb9", 25, "\r\n\u{3}고스트는 한번에 둘 이상 생산할수 없습니다.\r\n"),
        ("7e887e2b72a146becbdfa4832d30e1adbde9c9ed3ebe5294def6c8dda4232522", 1,  "마린키우기 Let It Snow"),
        ("7e887e2b72a146becbdfa4832d30e1adbde9c9ed3ebe5294def6c8dda4232522", 2,  "제가 처음로 제작하는 맵 입니다.\r\n맵 지형 제작 -Men- 감사합니다.\r\nMade By: Lucas Spia\r\nThanks to \r\n-Men- ,Mininii\r\n"),
        ("7e887e2b72a146becbdfa4832d30e1adbde9c9ed3ebe5294def6c8dda4232522", 3,  ""),
        ("7e887e2b72a146becbdfa4832d30e1adbde9c9ed3ebe5294def6c8dda4232522", 4,  "마린키우기 Snow 3.5 정식버전"),
        ("7e887e2b72a146becbdfa4832d30e1adbde9c9ed3ebe5294def6c8dda4232522", 5,  "적 데몬 3.5 정식버전"),
        ("7e887e2b72a146becbdfa4832d30e1adbde9c9ed3ebe5294def6c8dda4232522", 6,  "Snow 3.5 정식버전"),
    ];

    for (a, b, c) in test_vectors.into_iter() {
        assert_eq!(c, f(a.to_owned(), b).await);
    }
}
