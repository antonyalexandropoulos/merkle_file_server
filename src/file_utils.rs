use std::fs::File;
use std::io::Read;
use std::iter;
use std::path::Path;

const CHUNK_SIZE: usize = 1024;

fn pad_vec(data: &[u8]) -> Vec<u8> {
    let mut result = data.to_vec();
    while result.len() < CHUNK_SIZE {
        result.push(0u8);
    }
    result
}

pub fn pad_leaf_layer(data: &mut Vec<Vec<u8>>) {
    let next_power_of_two = get_next_power_of_two(data.len());
    while data.len() < next_power_of_two {
        let payload = iter::repeat(0u8).take(32).collect();
        data.push(payload);
    }
}

fn get_next_power_of_two(amount: usize) -> usize {
    let mut num = amount;
    num -= 1;
    num |= num >> 1;
    num |= num >> 2;
    num |= num >> 4;
    num |= num >> 8;
    num |= num >> 16;
    num += 1;
    num
}

pub fn split_file_to_chunks(filename: impl AsRef<Path>) -> Vec<Vec<u8>> {
    let mut file = File::open(filename).expect("no such file");
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer);

    buffer
        .chunks(CHUNK_SIZE)
        .map(|chunk| pad_vec(chunk))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_size() {
        let expect = 17;
        let chunks = split_file_to_chunks("test_data/icons_rgb_circle.png");

        assert_eq!(chunks.len(), expect);
    }

    #[test]
    fn test_next_power_of_two() {
        let expect = 32;
        let got = get_next_power_of_two(17);

        assert_eq!(got, expect);
    }

    #[test]
    fn test_pad_result() {
        let expect = 32;
        let mut chunks = split_file_to_chunks("test_data/icons_rgb_circle.png");
        pad_leaf_layer(&mut chunks);
        assert_eq!(chunks.len(), expect);
    }
}
