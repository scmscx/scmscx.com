use crate::minimap::TILESETS;

#[test]
fn test_load_all_tilesets() {
    assert_eq!(TILESETS.get(&0).unwrap().len(), 31664);
    assert_eq!(TILESETS.get(&1).unwrap().len(), 32736);
    assert_eq!(TILESETS.get(&2).unwrap().len(), 20240);
    assert_eq!(TILESETS.get(&3).unwrap().len(), 22688);
    assert_eq!(TILESETS.get(&4).unwrap().len(), 32736);
    assert_eq!(TILESETS.get(&5).unwrap().len(), 32736);
    assert_eq!(TILESETS.get(&6).unwrap().len(), 32624);
    assert_eq!(TILESETS.get(&7).unwrap().len(), 32752);
}

#[test]
fn test_minimap_rendering_badlands() {
    let mtxm = include_bytes!("test_vectors/badlands_mtxm");
    let bytes = include_bytes!("test_vectors/badlands_rendered.png");
    let view = unsafe { std::slice::from_raw_parts(mtxm.as_ptr() as *const u16, 16384) };
    let png = crate::render_minimap(view, 128, 128, 0).unwrap();
    // std::fs::write(
    //     "/home/stan/git/starcraft/crates/bwminimaprender/src/test_vectors/badlands_rendered2.png",
    //     &png,
    // )
    // .unwrap();
    assert_eq!(bytes, png.as_slice());
}

#[test]
fn test_minimap_rendering_jungle() {
    let mtxm = include_bytes!("test_vectors/jungle_mtxm");
    let bytes = include_bytes!("test_vectors/jungle_rendered.png");
    let view = unsafe { std::slice::from_raw_parts(mtxm.as_ptr() as *const u16, 32768) };
    let png = crate::render_minimap(view, 128, 256, 4).unwrap();
    // std::fs::write(
    //     "/home/stan/git/starcraft/crates/bwminimaprender/src/test_vectors/jungle_rendered2.png",
    //     &png,
    // )
    // .unwrap();
    assert_eq!(bytes, png.as_slice());
}

#[test]
fn test_calculate_perceptual_hash_badlands() {
    let bytes = include_bytes!("test_vectors/badlands_rendered.png");

    let ph16x16 = crate::calculate_perceptual_hash(bytes).unwrap();

    assert_eq!(
        [
            200, 131, 157, 131, 7, 0, 134, 64, 224, 32, 0, 0, 233, 28, 225, 30, 255, 14, 127, 0,
            248, 0, 240, 32, 96, 0, 124, 0, 254, 0, 248, 0
        ],
        ph16x16
    );
}

#[test]
fn test_calculate_perceptual_hash_jungle() {
    let bytes = include_bytes!("test_vectors/jungle_rendered.png");

    let ph16x16 = crate::calculate_perceptual_hash(bytes).unwrap();

    assert_eq!(
        [
            6, 208, 0, 104, 195, 16, 99, 0, 3, 15, 255, 255, 0, 0, 3, 192, 55, 252, 231, 231, 192,
            1, 252, 63, 199, 225, 255, 255, 1, 192, 255, 159
        ],
        ph16x16
    );
}

// #[test]
// fn badlands() {
//     let mtxm: [u16; 32768] = [1];

//     unsafe {
//         std::fs::write(
//             "jungle_mtxm",
//             std::slice::from_raw_parts(
//                 mtxm.as_slice().as_ptr() as *const u8,
//                 mtxm.as_slice().len() * 2,
//             ),
//         )
//         .unwrap();
//     }
// }

// #[test]
// fn png_file() {
//     let bytes = include_bytes!("test_vectors/jungle_mtxm");
//     let view = unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u16, 32768) };
//     let png = crate::render_minimap(view, 128, 256, 4).unwrap();
//     std::fs::write("out.png", png).unwrap();
// }
