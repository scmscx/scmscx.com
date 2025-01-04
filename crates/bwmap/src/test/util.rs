use anyhow::Result;
use async_stream::stream;
use futures::Stream;
use sha2::Digest;
use std::path::PathBuf;

// pub(crate) fn get_all_test_maps() -> impl Iterator<Item = DirEntry> {
//     let vec = WalkDir::new(format!("{}/test_vectors", env!("CARGO_MANIFEST_DIR")))
//         .into_iter()
//         .filter_map(Result::ok)
//         .filter(
//             |e| match e.file_name().to_string_lossy().to_string().as_str() {
//                 "[EUD]컴디 파이널.scx" => false,
//                 "마인의 폭피 1.scm" => false,
//                 _ => {
//                     !e.file_type().is_dir()
//                         && (e.file_name().to_string_lossy().ends_with(".scx")
//                             || e.file_name().to_string_lossy().ends_with(".scm"))
//                 }
//             },
//         )
//         .collect::<Vec<_>>();

//     assert_eq!(vec.len(), 186);

//     vec.into_iter()
// }

pub async fn get_chk(chk_hash: &str) -> Result<Vec<u8>> {
    let dir = PathBuf::from("/tmp/artifacts");
    tokio::fs::create_dir_all(&dir).await.unwrap();
    let path = dir.join(chk_hash);

    match tokio::fs::read(&path).await {
        Ok(data) => {
            if hash(data.as_slice()) != chk_hash {
            } else {
                return anyhow::Ok(data);
            }
        }
        Err(_) => {}
    }

    let url = format!("https://scmscx.com/api/chk/{}", chk_hash);
    let bytes = reqwest::get(url).await.unwrap().bytes().await.unwrap();

    assert!(bytes.len() > 0);
    assert_eq!(hash(&bytes[..]), chk_hash);

    tokio::fs::write(&path, &bytes[..]).await.unwrap();

    anyhow::Ok(bytes.to_vec())
}

pub(crate) fn get_all_test_chks() -> impl Stream<Item = Result<Vec<u8>>> {
    stream! {
        for chk in CHKS {
            yield anyhow::Ok(get_chk(chk).await?);
        }
    }
}

fn hash(bytes: &[u8]) -> String {
    let mut hasher = sha2::Sha256::new();

    hasher.update(bytes);

    format!("{:x}", hasher.finalize())
}

#[rustfmt::skip]
const CHKS: &[&str] = &[
    "ea537b0ce9ed0dfdd0c3c027e8cb10f47532734d1e54c8b767185348c0eb8451",
    "51e3017c89092f1d470a07a240bc59d545b7d10060c231b517c02686e7f1a3c4",
    "b3608e060c4fd739c5980d10a6d1faa530d550eb33a307e9ab15299b8379e0f4",
    "dbf2111db28861fb9d039d9ec14b038b2574f06196e5cc329f5d78b3deb6df8a",
    "dba0e1c4f9fd59ab087594b090c9d0b0c3eb50eca1dc3f3f5ae303b79891675a",
    "d12f9050eb0029756b763b2eb30a442acea627606d275714dca86664e15e3c92",
    "a604137418e7e66ec4306603f11c56c2f063a158495be6e07c657c3b2c059bcb",
    "28bfb92010f68f78da9e065b55f9bef6056050629ed1a565a2800edb39a54adf",
    "87f8873a87555a1d1f417e7cba6d7a54315eb2869a3af8f91fd6237c840e79d9",
    "da26253e96654b7030a930754dacb564a71105fd919fe3c7926f8d70eb76ab60",
    "13c1e290f984d42bd9bf5b42f9cde5de618925b5bc10833bfef0fac554678ece",
    "69468d54161919ab6a5222bc55a65fec55c869897b8adcb42e2d25b210816cde",
    "99eb979c6c676e6d7944b536563852ff8b15e09af9c1be4dad4132bb4c32f3b2",
    "fee4a9fdcb934075009ee74ec51f972be327d2bf935fd6f3a80c55530de82fe1",
    "40920d7d168089176069b55d234c76ec7765634626852f30afc99c76334f73dc",
    "14cf80213ba93e8001ea297e9e235f2130329a90c073d9486959e5f6cbb78d82",
    "53565118a4078abd83b0c81454ce52af8b9f0ce83eb39ea44a07e17514130664",
    "0c5753fcda21a5a6c59676882bf12b8c4647d2824bcb84fbf46c2465fa5a7cb0",
    "8eb39c0a24b1e6661c5ec98888ee3a47f5b02cb51ac26f72b0ef55f5b4f92a8a",
    "4b620ede87ec890ba8ad865d734e23f95b829667e930af9ba15fc304069d7d9b",
    "f143ad13aaaad95f7e18eecfdd2d611aed1eaafbe6f1f641b880a00adb3dcfb0",
    "79dadcb4ce5661166252939d11a21bccc7752b85fe76de680de313bccb9429db",
    "f64de5c141cb4a121b93acb65df057077214b119193b079bd82ab130374a2dbf",
    "f168123cc62046479eebc8f37a74220bb44466d1ead9381c5438c2d6bb17e20a",
    "e2e1458181a8e4c63ee5488fb951d7a153e7e04823220668a5e60b10940e5f28",
    "b7eeb58a98d0ead481e05a23c72a06215eca969e39a1eff75525b23386641de0",
    "1600b14ed9bf43832b6cfa59a47bb7983f9e471e495edc8c0041bb78497c33ef",
    "6fefdadaa34a0c727e9289ceb5dd20cc2f983b67cbd47878b601e1a7de4c37d0",
    "18daf88215fa1a348a532101146e43774e446efa267ae36f150b89d6d79dccca",
    "90a7cb6fd08537c056e55dc13f55f69211c6a8509b533de588dd5d38a4328187",
    "75a96caa3b9b851fa255e529cc4c9fe059bd672096e6e750a9d25cbc32f950a4",
    "35ca6b5bfa508cb4197b2158010baa1051f2ec231f14556552ca64db4aa1b5a4",
    "322994baf9d3c269e103cf56d48f9365bb6f980d6e3ff831f91c270d1c6c2781",
    "fd05963c147bb4ba687fc276b45b5f17590f0be8ea7d65fbfd649613532c8f0e",
    "0d230b884f955551a6a1cc46c2edc7a1c1e4952941c930daf3216f259c914e2a",
    "2bd414a929cbd17d83d203d54f4e1412e86089315234849931a0fb4655b79d46",
    "fb813182a86f4315b0a9c40da8f6d274d6c28d587ae753e3c217c7e0d8e22c2d",
    "5b1f012eba8af1ec0277fd673f76314dfc39a848193487d55cd9011ee92011f1",
    "786cb9dfee622e97cf41f12bcc1135bae483ceb4a9f7cafb33e810ae2b84b66b",
    "cbb3382256df137eecb61551efecc126f5d435265b25d86565d93c581f4c3990",
    "53d938c37cbb1b5959bea9675c6dc1309a2a9c3a657a0bff84f731162e7f9e13",
    "9f91c831906479ac12e892ebe39c8f05061a1db0a1c3d1d7e83dce9262b72aeb",
    "c768d196d020cbd8a765e3f7ea91a004c8b19b376e02ee88e604bc2aeaa1da57",
    "997663060cf54f7795f3f826c1babbff0284a9f70072865b5dfc4b1d4a14de6d",
    "e2f1a1ab56e83fd50956da06ffd617d96f43d6028664693cd9fa570dfa297539",
    "f53124b278533922a23ef770c3dd92c120e1e135f2ef5f60e0b3eabdfdb1639a",
    "be31bf2d903520bc5a5bf24cec8232be35ab175f55dca25f84f3af7479f4ef74",
    "1df3798bc911fe99f40fea11beb179a5ed34a851d0ac215efccd330cc676922f",
    "4df6ae2e450aa51c024f61cc0209410299b251b14c244d05589fca3f3d3e48cb",
    "f1848bdce403cef255c5a1395e04cf90fc56ac498cd6fab016eb9ee5859510cf",
    "816c2a15abc96f955c6483ebdbfaa4772544249c93616300a16c5c0634521cb1",
    "949c1aa77e77c6d7e165b0a50b3b4a4b1cabb3868d06bdc0fd529bf8a35f6bf8",
    "693637daa446c6206fc9d1910866bf1712c84cab619028145ed042f4985dfc89",
    "8101f8349ec0f68f89b5c568c5cd3d16ee81292d74267792fef9e333e0e794ae",
    "e1020e7169d92ffba63b44fa52f52ce8dd4281aaa0a27767518280ceb7b13d50",
    "7aab3f9ef08ad976ae7b50ab9dae993b101ce4d02834b78cfeee90fa567c67a1",
    "7abefe34f376d8dbc947bea03237c27ccbc00aa1e6fa12d3318c4c05a945fe8b",
    "2dbd93abe37df8cc7e01b464e4a670823d6b232f8bd9c8963e5d2a52d5c6fea2",
    "35106aeb9ea274a4b3e337c24cdd11a497082379d1bdf4167c7147062c3d9afc",
    "3ebca9c025d0681362c4d505f96ffbef3f2ab83f7ca4624db8f65bc63240e965",
    "509c6db72172c7b6f98f5c551d39e222de2708f6e00dbcb038a20ac152316189",
    "b7901d531d0463c014ec5544cc46c11e2f3a36bc7c4841dd0f8285164d549aee",
    "630b0617471e3b77d3f73a28b216ba3e53bc57569d488dd41504f254dc5ac808",
    "e60d9c0486893f468b0fb4720344abc623cd4974205eaa839a2992e8f42c7953",
    "e836f1a3402f7fb3136e0ea0bd4d2ac3ba78c9238d7f9505262cf1bd0af7d195",
    // "62a38d564a24dce92c979bbed2d0dfe2a96c41ab612c60aae8b476272ab072f4",
    "28003d1e7128945672a5a1bcdc4fbcae03288d24c51db2d8f3fcdffcf5264385",
    "427b6ebfbdc97bcfe7e2cadc8f98799cee01cd812bd69a106616e27335edc3f1",
    "8e5f8c6fae4d5ab1b88be1e41a7fb83b4a99bbb61e15d4d09f5aae22f8908dee",
    "d3e7310b02fc5f296299b6dd6c22f9850db879407f39c8244cb794a385505fa3",
    "fe2ce4497136151648c0ede1e019594aeb3982b03f93ef09f2c3f1d2fb5bca70",
    "a110aab71dd9a8befe2e792fd652c755f1e3233c5cc7da941a06d409badf1588",
    "b15021c695f4713272907c5a6c9df749677f747734fd6c57c52818197508d999",
    "6fb6644eff09ac0b028c026bbd4596db54823588be64d73d929c48f80430bd34",
    "7e887e2b72a146becbdfa4832d30e1adbde9c9ed3ebe5294def6c8dda4232522",
    "2306324577c785371ba07d44d8a0cffcf6becaace6360f34bf2725e6dec2ba0d",
    "c2cd74ffa02f3521edfa8f59a036953ea7e79d051e2820f2276aaee04da45d74",
    "6f10fe52079bec09b2073868cbb0c14f6844e60559b865d4a08d7926c7e9c0ef",
    "55c1dd5f83a75087594e4f271aeb36b671e41f7d0a4bf6c3cc31e0325638b344",
    "7f51c2920bbf70390669907fe47d59dc13d660fa8e130d7f1ad54d6177b598ba",
    "e0162456cf6b4bdd1eefa5b774567850f27d53bf8c8f89250cb4df01e2705a03",
    "3c41e3490cdd700c5bc9f5a230181fb2c99a049f372271a68a1e1f1d95f124a2",
    "6029abb367a7c0ba5b9f9dfcfff6f7d603af72e915156f84f04211d2e5301fac",
    "22b91b768c00d06f09db283405358a49c4cddd34c9c0e3a85f27bb070df2d8dc",
    "67dad5897f02dfad691d7f5098b1fc5726d6e8d09fe1af94f1ca88d890454a08",
    "313862696cd7f1672554afef8eb6095fabce833537005e01ff190df85a7be2fd",
    "e0b364765d1ae37159f81421d914dd9941c858a1c4173bd5cc7c4cf0aaf9ce32",
    "87f7b3210c28516d6934632020c2384c5a9cc95342c4ac80fb8bc82861d6844d",
    "1a07a3fa4fe644e290f0a8bfe747aa20da5c4b5d20f01dad6c92455bfb7e6803",
    "fe2ebb18b5b21e940a644fe22340e3f1bd55c85bdea4e76eff5aa53f0bc5993d",
    "05831066aae3355c3a7331caf794529c5f34979b82f869e78434c48492951479",
    "8a5a745e2a2ac2b4d6b30a0594870463acbab8bb9d1166a21234f1847a869dcf",
    "7c932724741a2505c5a1e18bb0df996e2be22014e3e69d75e1f91c980917a1d9",
    "0e638af6fa94eadae16849be5098614e524a3b6e3bbe93e011c3d64365f31851",
    "2ceee0a4dea3418b67a599639e452c1386e1e67661dc6bcd7ccd2fa4fff5a7f7",
    "ef876044a734c115275ca93bb658340d3d37e80031fa4fc93839ce6c2dac534c",
    "4f385fc467d1cd7069869b6a7ab1cd542dc8c98eec44d7ad3df57789372b5bb4",
    "283b6b542e5e13b893fa1f5f3eb4c6bb4f3d69aaf0e4bcc5e8c6afcebc55bdb8",
    "429a3bfaaa71daf89d36ae0965d7064d2d6a57475379954604c3b26143310fb3",
    "5718a76448937db314face2a66367713a7c030612f5fdc1643021e21905c921c",
    "89822d354b28e9389ab7468d2abc200eeb94e8e589ce087725e99279fa619f86",
    "3ad31659173ab82fb3580eb3ea952b19986a953414199603b33a561a221edbbb",
    "5e65dd5183933d250e27a503f9202e5525322fb637f107ff3f33c87f8c477632",
    "6d4662d96c71500031c904c40a93b90d5ea03de5e3c277e08471168462e7113e",
    "c7f514521a17acbd128724a5daedcc317a3841564b0774780589330536bc29d3",
    "aa0ed8c257ab6dbcfa2afc3ddf5f54c7dbc886dd91d49783c3995946ab14e544",
    "2602365f09bebe0d1cb4d7203b45df89ab5f4c3966735dbd8a9376f53701988d",
    "0c9f00fa65fb2977b1e5df4e160d90d5f26946f39bce66122d441b2d6ee59467",
    "13e328f170aa38af1e6cd8f3245734cf18cdd815547acd351f54778f828aef54",
    "5b4fbab16da0018765c3f5b1120171d1f8c4145f14792af51bbfd27290f1cf38",
    "dea023a79647ce61cc2ac0c43437832de8f78a12c94e330ef9969c43448d9352",
    "e836f1a3402f7fb3136e0ea0bd4d2ac3ba78c9238d7f9505262cf1bd0af7d195",
    "7ed07e6f8e09f4f12f5c71e281e05439c5fb599ebe7013591b4fcc7bea36d83c",
    "d73239245f50f10c6b16a632033fb6e6ba098bd04c8ef9bae1a01c7357126e78",
    "5cf55e4ec2e2a7c510e6cbdcf4ade3299a273f9db36aab655013ef468511b1d9",
    "771ee3d93c41a61e047192add21348def9e3ae29124c0eb2d4121ee34de9aef4",
    "bd7b3e491caf2d239a1dfc6aa0f9a74bd8064393c8467411ff4e9aaac10449c8",
    "10437773ead1b4beb5f4cb003f3cfdcd117b489d588933936e11fb3298cde479",
    "e274bbc3543e6a82937bde3c7589d7db3626ab2b9c0c0d8a1e836ef7e65202c1",
    "809dcb193c33afda0df0841818869b791dc0031f55bf28d0d55bf8f27c2949c0",
    "1109de056098352faa35574c8ac266cb94bb1790fed85305a3fda4d99065365b",
    "92e6b3c30d6bdecc3063627e9cc69cb35e5f1ac60ae1322b8914ed7897718f3c",
    "767bc878e7b800c1c25cb16bef47413a5623af81429263fd8696e6b5071ff240",
    "96cd0ccfd0cd89b9fe4e9bb68e7548645e74e40a92ed40637c11f185fff49815",
    "674ff4dca98421766cf508b83d38424c6d421c539e7dc17462d036665157b605",
    "20de1c9c57fdc41ac20c6f19a4d231f32c6dd313e2de09c2ed84042f4d9e735d",
    "50193934b6ca0d015e5b41764ce18483348976b3e49bea205904b9cd2c4258be",
    "22000aae8545fa3e73d7c10c3d5ee14d33580aadaebc32617db23bc140b79319",
    "73035db5ac3391d905b9b6c239e94fe2ac238b0e51cec307f940a78b76e6e11b",
    "8b8dcb0b126a97e970583cbe8e20ac3871ee20854ebcc20ee1e978a2f7d46d6b",
    "41af2580f4c1681ecd54216ac54547f8ba4f230ed9462c651e180f6d036a794d",
    "d921165a300b6ebb2df5f6995c714b9c867f7ac1dd19c503007f61f8b9b14b58",
    "80acbc1c5c9fe4cab274c191f818c2ae911fd1dec37636b3855fb6a35680ba97",
    "9e4e5285188131b4f3bbf0e2f2eb21744649788c5519206c9ecf4e998307e0f3",
    "0341684d925a364412113a11614bdb19f65bfae24c6dbef67de13b6922af5a22",
    "8b970227ecf8ea89eadf7cfd58ee7908aaffe6124f45d39b1c0b83020fb07c7c",
    "edbeb33118923ced0f5305b5bca1d5bf5e94427f44a2d7cbf3b40d26840ed2f5",
    "bb2d1dd37853f60d35b372b4280e80956407aa1be21c6fd89275a18a0edc15f5",
    "14d34632b03f2c929fd4e63349f69755d14cfddd29af910ba3adfef45f37730f",
    "7c4de46e15e4e6de898d94636db4f3ad235bc60b3bfc7a066b4099a9c099d0d1",
    "de59af0c219e8b92f31d38bb2ff75bc7595828fd86c208a763aa0e1383057a00",
    "ef32c5707243bccb588a92d6180274f09597bfdae7b0aec1316f5b22f7b5bd74",
    "1b7dd95b26d23205dff5ce2e812c3e7cc601509cb9f9c3474c16f5f7e0ccbbf6",
    "c46d4e76bc2eea3b4a817557fb0467ab4afd0e1085e2713a71f54167cbe17631",
    "7efb6ee8c4a2e2d1f9292f2022515979d22bd385e23ca1b3cafa59446ee579d0",
    "8a712b57a022b547c0c864660dff39cf20956edbc879dd2cafb3a75add2dd9bb",
    "2695626b4d3e470db9dd8f72968721a24efc41d026bf22c5d3e278cdd814ff87",
    "9b3b7e3c62b84021008be15626001c4da3b8c13c5118037861bfa4c94566b468",
    "f7d5223d73593236573acd3d8f398c3c6fbc7686bede6cc086c159f1592c16c0",
    "5947eaede71b908b8d25202f8792fd8a87e1c08dc304871cc6a4ede0a08b6ee7",
    "3ed5e7d99d86292c46170baf4b869bd71dae515831f8079cfa857ef8d7264b37",
    "ec3e28f75ed0b81b616caacc4b7d0d663853daf073a000ee3ea789ee7ae103f1",
    "a892ed9f54bb7624b3fbb13e7005fafaeeea56b83e9b7767aeb8209a0bbdb902",
    "0858b5e157344d80ee0d250c8189c41d52d9f91481838ee88c2d948fff0006d2",
    "0c3433419ddfa8f251fd50d1dfee6699c32b7063229fa29f05fa561b3cd461e5",
    "595854eaf1d343515fac49b6d595d09c063cd5c5cc96933c48e56a60f802573d",
    "146970e32ba19d5638de167dcd720e0ec27f83b3a154ddd46c843248e79436c4",
    "5ee1dd4e4606799b6bcb332b3b2fcb924a6474ac325cc054de2dadfe789e36b3",
    "247e38b5eb847782f3e40f27bb8db0e3b1f9e86307cc2b5b21d9a273b12693ae",
    "8af9b0b1951deac6d1471346cce4af189a6eae569e55f850061b0f2d21619c0a",
    "541bb96fdd38d14a6dc2cd877fa80d480e2e52ccd96e76e17e276deac4d23f52",
    "673bf2dd39218b9a14eec9e4e254958618ae5e46db175fc06d4d8d0400f145c7",
    "4b48e5e719e455ce02015e2af9a8f277bcef5295784ca7d352861b526281d7a4",
    "d434b39cd014495a349024c190e480be76a14d7fbcb8a2ae60d69985ce477e71",
    "e6d9563dc48df444c92d91aef3bee7286e8683a6846e3ab3cd94519cdf9f9652",
    "5dcb5fcdf1b46ca8840cdc547218d3e2625dddb67806f1158e75ea9e6da81105",
    "dd3acc72fd6b11f4d545581342670cda1f3f42b871191dd0cb8e3db762ba4a9d",
    "258333b146cea1d8af5e853684c68db2c269c378cbba41d6d3b7d7797c74c926",
    "a7c4d78dfa4be4b115a6e1a4109bf570a5ff245360e02e3086ea87c20663fe13",
    "4cdb617ace6b785c9c7a3999530f650f1f0423974bd93056cd88d701ec5fd151",
    "1d901afa80c7cf8647a35c1d4cc2f5baa8a81928efdc860e169e50f5e77d7f8d",
    "17e9624d732cc460ea1bea7873bb2cfa0a98ba0cbde366a1396c96116bfd676b",
    "c796eb5aa0b25adaf833e9be9c6ced136cbb73764ccb07c089d8dc51988f880d",
    "c34b6a16015217a072182d5a1dd6b1f131c3bbe91f65d5916c1099848529c59e",
    "e53c788dc91f55902a45c3df357f0571900f1503fab20a84bda0abefbf882e5d",
    "fd59eb2e73452bc02d0e4a0aee089e2d68986eb040173baefa7dca7cfc7d9940",
    "bdde7d9c76f0bf8bc55a2f478c07b84b4341736f08952bd9fa4b97ef960246ff",
    "5df8755230b337fa02bdd8ef6e9cc309b62110948d17b79abde63b318392812b",
    "b828d46e456a8f43ec893e95428fe7e9b88651372c0cace1b0fef962376c2be9",
    "58a66db4ef8e66b45e2c9f0eb558607b549d84dd42b7d4c464d045da86110069",
    "1cd434fbee9f855cb443ec218831d1472a1b4373bfe088556767e781adee8924",
    "1eb9cf3750bf5cbdbe7a67c1cb7b47424be441f22a4ba504c42d7c6026220fe0",
    "0eebcf2e44ad4384e7ebc421230ac98a1749e92ab823e0fbefc341281cb2022a",
    "2c9eafcc3a5cf88a7c7761af948e3bcd296f69904e3a7b0db0f77233af79096a",
    "03e5eaf424ef0d303b9a7a4e043c27a169f71b977fda71040dffc97c4c750d57",
    "b69c56d44a60f41bf08b6c3d8dcbf30bc1d1c0bbbe806be8a98866cd5694607e",
    "fbf8b79263397e41c48cfab7141d91cb71e8e771cbda3bd7bd69b838a5eb0128",
];

// async fn process_iter_async_concurrent<I, T, F, J, R, F2, H, Z>(
//     mut iter: I,
//     cloner: H,
//     max_outstanding: usize,
//     on_item_completed: F2,
//     func: F,
// ) -> usize
// where
//     I: Iterator<Item = T>,
//     F: Fn(Z, T) -> R,
//     R: Future<Output = J> + Send,
//     F2: Fn(usize, J),
//     H: Fn() -> Z,
// {
//     let mut futs = Vec::new();
//     let mut counter = 0;
//     loop {
//         while futs.len() < max_outstanding {
//             if let Some(entry) = iter.next() {
//                 futs.push(func(cloner(), entry).boxed());
//             } else {
//                 break;
//             }
//         }

//         if futs.len() == 0 {
//             break;
//         }

//         let (item, _, remaining_futures) = select_all(futs).await;

//         futs = remaining_futures;

//         counter += 1;

//         on_item_completed(counter, item);
//     }

//     counter
// }

// pub struct ChkCache {
//     root: PathBuf,
// }

// impl ChkCache {
//     pub fn new(root: PathBuf) -> Self {
//         Self { root }
//     }

//     pub async fn get_chk(&self, chk_hash: &str) -> Result<Vec<u8>> {
//         let path = self.root.join(chk_hash);

//         match tokio::fs::read(&path).await {
//             Ok(data) => {
//                 if hash(data.as_slice()) != chk_hash {
//                 } else {
//                     return anyhow::Ok(data);
//                 }
//             }
//             Err(_) => {}
//         }

//         let url = format!("https://scmscx.com/api/chk/{}", chk_hash);
//         let bytes = reqwest::get(url).await.unwrap().bytes().await.unwrap();

//         assert!(bytes.len() > 0);
//         assert_eq!(hash(&bytes[..]), chk_hash);

//         tokio::fs::write(&path, &bytes[..]).await.unwrap();

//         anyhow::Ok(tokio::fs::read(&path).await?)
//     }
// }

// async fn download_test_artifacts<'a, T: AsRef<Path>, I: Iterator<Item = &'a str>>(
//     dir: T,
//     iter: I,
// ) -> Result<()> {
//     tokio::fs::create_dir_all(&dir).await?;

//     process_iter_async_concurrent(
//         iter,
//         || dir.as_ref().clone(),
//         8,
//         |_x, _y| {},
//         |path, id| async move {
//             let path = path.join(id);

//             match tokio::fs::read(&path).await {
//                 Ok(data) => {
//                     if hash(data.as_slice()) != id {
//                     } else {
//                         return;
//                     }
//                 }
//                 Err(_) => {}
//             }

//             let url = format!("https://scmscx.com/api/chk/{}", id);
//             let bytes = reqwest::get(url).await.unwrap().bytes().await.unwrap();

//             assert!(bytes.len() > 0);
//             assert_eq!(hash(&bytes[..]), id);

//             tokio::fs::write(path, &bytes[..]).await.unwrap();
//         },
//     )
//     .await;

//     anyhow::Ok(())
// }
