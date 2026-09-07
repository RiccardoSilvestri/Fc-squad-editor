const POLY: i32 = 0x04C1_1DB7;

pub fn compute_crc_db11(bytes: &[u8]) -> i32 {
    let mut num: i32 = -1;
    for &byte in bytes {
        num ^= (byte as i32) << 24;
        for _ in 0..8 {
            if num >= 0 {
                num = num.wrapping_mul(2);
            } else {
                num = num.wrapping_mul(2);
                num ^= POLY;
            }
        }
    }
    num
}

pub fn compute_crc32(bytes: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in bytes {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_is_all_ones() {
        assert_eq!(compute_crc_db11(&[]), -1);
    }

    #[test]
    fn crc32_matches_known_vectors() {
        assert_eq!(compute_crc32(b""), 0x0000_0000);
        assert_eq!(compute_crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(
            compute_crc32(b"The quick brown fox jumps over the lazy dog"),
            0x414F_A339
        );
    }
}
