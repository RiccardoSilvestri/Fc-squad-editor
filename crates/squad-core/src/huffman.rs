pub struct HuffmanTree {
    child: Vec<[u8; 2]>,
    leaf: Vec<[u8; 2]>,
}

impl HuffmanTree {
    pub fn load(buf: &[u8], start: usize, n_nodes: usize) -> Self {
        let max_nodes = buf.len().saturating_sub(start) / 4;
        let n_nodes = n_nodes.min(max_nodes);
        let mut child = vec![[0u8; 2]; n_nodes];
        let mut leaf = vec![[0u8; 2]; n_nodes];
        for i in 0..n_nodes {
            let p = start + i * 4;
            child[i][0] = buf[p];
            leaf[i][0] = buf[p + 1];
            child[i][1] = buf[p + 2];
            leaf[i][1] = buf[p + 3];
        }
        HuffmanTree { child, leaf }
    }

    pub fn decode(&self, buf: &[u8], start: usize, out_len: usize) -> Vec<u8> {
        if self.child.is_empty() {
            let safe_start = start.min(buf.len());
            let safe_end = (start.saturating_add(out_len)).min(buf.len());
            return buf[safe_start..safe_end].to_vec();
        }
        let mut out = Vec::with_capacity(out_len);
        let mut node = 0usize;
        let mut byte_pos = start;
        'outer: while out.len() < out_len {
            if byte_pos >= buf.len() {
                break;
            }
            let b = buf[byte_pos];
            byte_pos += 1;
            for bit_index in (0..8).rev() {
                let bit = ((b >> bit_index) & 1) as usize;
                let next_node_idx = self.child[node][bit];
                if next_node_idx == 0 {
                    out.push(self.leaf[node][bit]);
                    if out.len() == out_len {
                        break 'outer;
                    }
                    node = 0;
                } else {
                    let next = next_node_idx as usize;
                    node = if next < self.child.len() { next } else { 0 };
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falls_back_to_raw_bytes_when_tree_is_empty() {
        let tree = HuffmanTree::load(&[], 0, 0);
        let buf = b"hello".to_vec();
        assert_eq!(tree.decode(&buf, 0, 5), b"hello".to_vec());
    }

    #[test]
    fn load_caps_n_nodes_to_available_data() {
        let buf = vec![0u8; 8];
        let tree = HuffmanTree::load(&buf, 0, 1000);
        assert_eq!(tree.child.len(), 2);
    }

    #[test]
    fn decode_stops_at_eof_not_panics() {
        let buf = vec![0u8; 0];
        let tree = HuffmanTree {
            child: vec![[0, 0]; 4],
            leaf: vec![[b'a', b'b']; 4],
        };
        let result = tree.decode(&buf, 0, 10);
        assert!(result.is_empty());
    }

    #[test]
    fn decode_raw_fallback_clamps_to_buf_len() {
        let buf = vec![1u8, 2, 3];
        let tree = HuffmanTree::load(&[], 0, 0);
        let result = tree.decode(&buf, 1, 100);
        assert_eq!(result, vec![2, 3]);
    }
}
