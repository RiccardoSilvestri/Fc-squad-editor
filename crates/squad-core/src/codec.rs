pub fn read_field_bits(record: &[u8], bit_offset: u32, depth: u32) -> u32 {
    let mut value: u32 = 0;
    for i in 0..depth {
        let b = bit_offset + i;
        let byte = record[(b >> 3) as usize];
        let bit = (byte >> (b & 7)) & 1;
        value |= (bit as u32) << i;
    }
    value
}

pub fn write_field_bits(record: &mut [u8], bit_offset: u32, depth: u32, value: u32) {
    for i in 0..depth {
        let b = bit_offset + i;
        let byte_idx = (b >> 3) as usize;
        let bit_idx = b & 7;
        let bit_val = (value >> i) & 1;
        if bit_val == 1 {
            record[byte_idx] |= 1 << bit_idx;
        } else {
            record[byte_idx] &= !(1 << bit_idx);
        }
    }
}

pub fn max_value_for_depth(depth: u32) -> u32 {
    if depth >= 32 {
        u32::MAX
    } else {
        (1u32 << depth) - 1
    }
}

pub fn read_null_padded_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).to_string()
}

pub fn write_null_padded_string(dest: &mut [u8], value: &str) {
    let bytes = value.as_bytes();
    let len = bytes.len().min(dest.len());
    dest[..len].copy_from_slice(&bytes[..len]);
    for b in &mut dest[len..] {
        *b = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_roundtrip() {
        let mut record = vec![0u8; 4];
        write_field_bits(&mut record, 3, 7, 87);
        let value = read_field_bits(&record, 3, 7);
        assert_eq!(value, 87);
    }

    #[test]
    fn bit_write_does_not_disturb_neighbours() {
        let mut record = vec![0u8; 4];
        write_field_bits(&mut record, 0, 4, 0b1111);
        write_field_bits(&mut record, 4, 4, 0b0000);
        assert_eq!(read_field_bits(&record, 0, 4), 0b1111);
        assert_eq!(read_field_bits(&record, 4, 4), 0);
    }
}
