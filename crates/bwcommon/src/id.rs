use anyhow::anyhow;
use anyhow::Result;
use crc::Crc;
use phf::phf_map;

pub fn get_web_id_from_db_id(db_id: i64, seed: u8) -> Result<String> {
    anyhow::Ok(encode_base32(&obfuscate(
        convert_id_to_byte_array(db_id)?,
        seed,
    )))
}

pub fn get_db_id_from_web_id(web_id: &str, seed: u8) -> Result<i64> {
    anyhow::Ok(convert_byte_array_to_id(deobfuscate(
        decode_base32(web_id)?,
        seed,
    ))?)
}

fn obfuscate<const N: usize>(mut bytes: [u8; N], seed: u8) -> [u8; N] {
    for _ in 1..13 {
        for j in 0..N {
            for i in 0..N {
                if i == j {
                    continue;
                }
                bytes[i] ^= bytes[j]
                    .overflowing_add(97)
                    .0
                    .rotate_left(5)
                    .overflowing_add(seed)
                    .0;
            }
        }
    }

    bytes
}

fn deobfuscate<const N: usize>(mut bytes: [u8; N], seed: u8) -> [u8; N] {
    for _ in (1..13).rev() {
        for j in (0..N).rev() {
            for i in (0..N).rev() {
                if i == j {
                    continue;
                }
                bytes[i] ^= bytes[j]
                    .overflowing_add(97)
                    .0
                    .rotate_left(5)
                    .overflowing_add(seed)
                    .0;
            }
        }
    }

    bytes
}

const USB_CRC_ALGORITHM: Crc<u8> = Crc::<u8>::new(&crc::CRC_5_USB);

fn convert_id_to_byte_array(input: i64) -> Result<[u8; 5]> {
    anyhow::ensure!(input >= 0);
    anyhow::ensure!(input < 1 << (32 + 3));

    let mut input = [
        (input % 256) as u8,
        (input / 256 % 256) as u8,
        (input / 256 / 256 % 256) as u8,
        (input / 256 / 256 / 256 % 256) as u8,
        (input / 256 / 256 / 256 / 256 % 256) as u8,
    ];

    let checksum = USB_CRC_ALGORITHM.checksum(&input);
    anyhow::ensure!(checksum < 2 << 5);

    input[4] |= checksum << 3;

    anyhow::Ok(input)
}

fn convert_byte_array_to_id(mut input: [u8; 5]) -> Result<i64> {
    let checksum = (input[4] & 0b11111000) >> 3;
    input[4] &= 0b00000111;

    let checksum2 = USB_CRC_ALGORITHM.checksum(&input);
    anyhow::ensure!(checksum == checksum2);

    let mut ret = 0i64;

    ret += input[4] as i64 * 256 * 256 * 256 * 256;
    ret += input[3] as i64 * 256 * 256 * 256;
    ret += input[2] as i64 * 256 * 256;
    ret += input[1] as i64 * 256;
    ret += input[0] as i64;

    anyhow::Ok(ret)
}

fn encode_base32(bytes: &[u8; 5]) -> String {
    #[rustfmt::skip]
    const CHARACTER_MAP: [char; 32] = [
        '2', '3', '4', '5', '6', '7', '8', '9',
        'b', 'c', 'd', 'f', 'g', 'h', 'j', 'k', 'p', 'q', 'r', 's', 't', 'x', 'y', 'z',
        'B', 'D', 'G', 'H', 'R', 'T', 'V', 'Y',
    ];

    #[rustfmt::skip]
    let encoded = String::from_iter([
        ((bytes[0] & 0b11111000) >> 3),
        ((bytes[0] & 0b00000111) << 2) |
        ((bytes[1] & 0b11000000) >> 6),
        ((bytes[1] & 0b00111110) >> 1),
        ((bytes[1] & 0b00000001) << 4) |
        ((bytes[2] & 0b11110000) >> 4),
        ((bytes[2] & 0b00001111) << 1) |
        ((bytes[3] & 0b10000000) >> 7),
        ((bytes[3] & 0b01111100) >> 2),
        ((bytes[3] & 0b00000011) << 3) |
        ((bytes[4] & 0b11100000) >> 5),
        (bytes[4] & 0b00011111),
    ].into_iter().map(|x| CHARACTER_MAP[x as usize]));

    encoded
}

fn decode_base32(string: &str) -> Result<[u8; 5]> {
    static INVERSE_CHARACTER_MAP: phf::Map<char, u8> = phf_map! {

        '2' => 0u8,
        '3' => 1u8,
        '4' => 2u8,
        '5' => 3u8,
        '6' => 4u8,
        '7' => 5u8,
        '8' => 6u8,
        '9' => 7u8,

        'b' => 8u8,
        'c' => 9u8,
        'd' => 10u8,
        'f' => 11u8,
        'g' => 12u8,
        'h' => 13u8,
        'j' => 14u8,
        'k' => 15u8,
        'p' => 16u8,
        'q' => 17u8,
        'r' => 18u8,
        's' => 19u8,
        't' => 20u8,
        'x' => 21u8,
        'y' => 22u8,
        'z' => 23u8,

        'B' => 24u8,
        'D' => 25u8,
        'G' => 26u8,
        'H' => 27u8,
        'R' => 28u8,
        'T' => 29u8,
        'V' => 30u8,
        'Y' => 31u8,
    };

    anyhow::ensure!(string.len() == 8);
    let mut iter = string.chars();
    let mut output = [0u8; 5];

    let chunk = *INVERSE_CHARACTER_MAP
        .get(&iter.next().unwrap())
        .ok_or(anyhow!("invalid character"))?;
    output[0] |= (chunk << 3) & 0b11111000;

    let chunk = *INVERSE_CHARACTER_MAP
        .get(&iter.next().unwrap())
        .ok_or(anyhow!("invalid character"))?;
    output[0] |= (chunk >> 2) & 0b00000111;
    output[1] |= (chunk << 6) & 0b11000000;

    let chunk = *INVERSE_CHARACTER_MAP
        .get(&iter.next().unwrap())
        .ok_or(anyhow!("invalid character"))?;
    output[1] |= (chunk << 1) & 0b00111110;

    let chunk = *INVERSE_CHARACTER_MAP
        .get(&iter.next().unwrap())
        .ok_or(anyhow!("invalid character"))?;
    output[1] |= (chunk >> 4) & 0b00000001;
    output[2] |= (chunk << 4) & 0b11110000;

    let chunk = *INVERSE_CHARACTER_MAP
        .get(&iter.next().unwrap())
        .ok_or(anyhow!("invalid character"))?;
    output[2] |= (chunk >> 1) & 0b00001111;
    output[3] |= (chunk << 7) & 0b10000000;

    let chunk = *INVERSE_CHARACTER_MAP
        .get(&iter.next().unwrap())
        .ok_or(anyhow!("invalid character"))?;
    output[3] |= (chunk << 2) & 0b01111100;

    let chunk = *INVERSE_CHARACTER_MAP
        .get(&iter.next().unwrap())
        .ok_or(anyhow!("invalid character"))?;
    output[3] |= (chunk >> 3) & 0b00000011;
    output[4] |= (chunk << 5) & 0b11100000;

    let chunk = *INVERSE_CHARACTER_MAP
        .get(&iter.next().unwrap())
        .ok_or(anyhow!("invalid character"))?;
    output[4] |= chunk & 0b00011111;

    anyhow::Ok(output)
}

#[cfg(test)]
mod test {
    use super::convert_byte_array_to_id;
    use super::convert_id_to_byte_array;
    use super::decode_base32;
    use super::deobfuscate;
    use super::encode_base32;
    use super::get_db_id_from_web_id;
    use super::get_web_id_from_db_id;
    use super::obfuscate;
    use quickcheck::TestResult;
    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;
    use std::collections::HashSet;

    #[derive(Debug, Clone)]
    struct MyStruct {
        data: [u8; 5],
    }

    impl Arbitrary for MyStruct {
        fn arbitrary(g: &mut Gen) -> MyStruct {
            MyStruct {
                data: [
                    u8::arbitrary(g),
                    u8::arbitrary(g),
                    u8::arbitrary(g),
                    u8::arbitrary(g),
                    u8::arbitrary(g),
                ],
            }
        }
    }

    #[quickcheck]
    fn can_encode_decode_and_get_same_data_back(input: MyStruct) {
        assert_eq!(
            decode_base32(&encode_base32(&input.data)).unwrap(),
            input.data
        )
    }

    #[quickcheck]
    fn can_obfuscate_deobfuscate_and_get_same_data_back(input: MyStruct, seed: u8) {
        assert_eq!(deobfuscate(obfuscate(input.data, seed), seed), input.data)
    }

    #[quickcheck]
    fn convert_deconvert_and_get_same_id_back(input: i64) -> TestResult {
        if input < 0 || input >= (1 << (32 + 3)) {
            return TestResult::discard();
        }

        assert_eq!(
            convert_byte_array_to_id(convert_id_to_byte_array(input).unwrap()).unwrap(),
            input
        );

        TestResult::passed()
    }

    #[quickcheck]
    fn get_web_id_from_db_id_and_get_same_id_back(input: i64, seed: u8) -> TestResult {
        if input < 0 || input >= (1 << (32 + 3)) {
            return TestResult::discard();
        }

        assert_eq!(
            get_db_id_from_web_id(&get_web_id_from_db_id(input, seed).unwrap(), seed).unwrap(),
            input
        );

        TestResult::passed()
    }

    #[quickcheck]
    fn web_ids_edge_cases(seed: u8) {
        let successful = [0, (1 << 35) - 1];

        for case in successful {
            assert_eq!(
                get_db_id_from_web_id(&get_web_id_from_db_id(case, seed).unwrap(), seed).unwrap(),
                case
            );
        }

        for case in [1 << 35, -1, 1 << (35 + 1), 1 << 36] {
            assert_eq!(get_web_id_from_db_id(case, seed).is_err(), true);
        }
    }

    #[quickcheck]
    fn web_ids_do_not_repeat_small(seed: u8) {
        let mut set = HashSet::new();
        for i in 0..10_000 {
            assert_eq!(set.insert(get_web_id_from_db_id(i, seed).unwrap()), true);
        }
    }

    #[test]
    #[ignore]
    fn bazoopy() {
        // println!("39807: {}", get_web_id_from_db_id(39807, 97).unwrap());
        println!(
            "9z8gR5dp: {}",
            get_db_id_from_web_id(&"9z8gR5dp".to_owned(), 97).unwrap()
        );

        assert!(false);
    }

    #[test]
    #[ignore = "test consumes too much memory, good to run though if the id algo is changed"]
    fn web_ids_do_not_repeat() {
        let mut set = HashSet::new();
        for i in 0..1_000_000_000 {
            assert_eq!(set.insert(get_web_id_from_db_id(i, 32).unwrap()), true);
        }
    }
}
