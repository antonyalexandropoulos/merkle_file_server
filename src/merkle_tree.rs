use crate::{pad_leaf_layer, split_file_to_chunks};
use hex;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};
use std::iter;
use std::path::Path;

type Node = Vec<u8>;

#[derive(Debug, Clone)]
pub struct MerkleTree {
    pub(crate) nodes: Vec<Node>,
    pub(crate) total_non_empty_pieces: usize,
    pub(crate) total_nodes: usize,
    pub(crate) piece_data: HashMap<usize, String>,
}

fn hash_leaf(data: &Vec<u8>) -> Node {
    let mut hasher = Sha256::new();
    let zero_value: Vec<u8> = iter::repeat(0u8).take(32).collect();

    if *data != zero_value {
        hasher.update(data);

        let mut result = hasher.finalize();
        Node::from(result.as_mut_slice())
    } else {
        data.clone()
    }
}

fn populate_tree(data: &mut Vec<Vec<u8>>, pieces_length: &usize) {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    (0..*pieces_length)
        .collect::<Vec<usize>>()
        .iter()
        .for_each(|index| queue.push_back(index + pieces_length));

    while let Some(index) = queue.pop_front() {
        if visited.contains(&index) {
            continue;
        }
        visited.insert(index);

        if 2 * index < data.len() && 2 * index + 1 < data.len() {
            let mut left = data[2 * index].clone();
            let mut right = data[2 * index + 1].clone();
            let mut hasher = Sha256::new();
            left.append(&mut right);
            hasher.update(left);
            let mut result = hasher.finalize();
            data[index] = Node::from(result.as_mut_slice());
        }
        if index / 2 > 0 {
            queue.push_back(index / 2);
        }
    }
}

fn get_uncle(child_node: usize) -> Option<usize> {
    let parent = child_node / 2;

    let grandparent = parent / 2;
    if parent == 0 || grandparent == 0 {
        return None;
    }
    if parent % 2 == 0 {
        return Some(grandparent * 2 + 1);
    } else {
        return Some(grandparent * 2);
    }
}

fn get_sibling(node: usize) -> usize {
    return if node % 2 == 0 { node + 1 } else { node - 1 };
}

impl MerkleTree {
    pub fn new(file_name: impl AsRef<Path>) -> Self {
        let mut pieces = &mut split_file_to_chunks(file_name);
        let total_non_empty_pieces = pieces.len();
        pad_leaf_layer(&mut pieces);
        let total_nodes = 2 * pieces.len() - 1;
        let mut result_data = vec![Node::new(); total_nodes + 1];
        let mut base64_map = HashMap::new();
        let leaf_layer_length = pieces.len();
        for i in 0..total_non_empty_pieces {
            base64_map.insert(i, base64::encode(&pieces[i]));
        }
        pieces
            .into_iter()
            .enumerate()
            .for_each(|(i, piece)| result_data[i + leaf_layer_length] = hash_leaf(piece));

        populate_tree(&mut result_data, &pieces.len());

        MerkleTree {
            nodes: result_data,
            total_non_empty_pieces,
            total_nodes,
            piece_data: base64_map,
        }
    }

    pub fn uncle_traversal(&self, piece_number: usize) -> Option<Vec<String>> {
        if piece_number > self.total_non_empty_pieces - 1 {
            return None;
        }
        let mut result = Vec::new();
        let mut node = self.nodes.len() / 2 + piece_number;

        while let Some(uncle) = get_uncle(node) {
            result.push(hex::encode(&self.nodes[uncle]));
            node = uncle;
        }
        Some(result)
    }

    pub fn proof(&self, piece_number: usize) -> Option<Vec<String>> {
        if piece_number > self.total_non_empty_pieces {
            return None;
        }
        let node_index = self.nodes.len() / 2 + piece_number;
        let mut proof = Vec::new();

        proof.push(hex::encode(&self.nodes[get_sibling(node_index)]));
        if let Some(uncles) = &self.uncle_traversal(piece_number) {
            proof.extend(uncles.iter().cloned());
        }

        Some(proof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_uncle_root() {
        let expect = None;
        let uncle = get_uncle(1);

        assert_eq!(uncle, expect);
    }

    #[test]
    fn test_uncle_bottom_layer() {
        let expect = Some(7);
        let uncle = get_uncle(12);

        assert_eq!(uncle, expect);
    }

    #[test]
    fn test_uncle_uncle_bottom_layer() {
        let expect = Some(2);
        let uncle = get_uncle(6);

        assert_eq!(uncle, expect);
    }

    #[test]
    fn test_uncle_uncle_bottom_layer2() {
        let expect = Some(2);
        let uncle = get_uncle(7);

        assert_eq!(uncle, expect);
    }

    #[test]
    fn test_uncle_uncle_bottom_layer3() {
        let expect = Some(7);
        let uncle = get_uncle(13);

        assert_eq!(uncle, expect);
    }

    #[test]
    fn test_tree_from_file() {
        let tree = get_icons_rgb_circle_tree();

        assert!(tree.total_nodes > 0);
        assert!(tree.nodes.len() > 0);
        assert_eq!(tree.nodes.len(), 64);
    }

    #[test]
    fn test_uncle_traversal() {
        let tree = get_icons_rgb_circle_tree();
        let result = tree.uncle_traversal(8);

        assert_eq!(
            result.unwrap(),
            vec![
                "956bf86d100b2f49a8d057ebafa85b8db89a0f19d5627a1226fea1cb3e23d3f3",
                "04284ddea22b003e6098e7dd1a421a565380d11530a35f2e711a8dd2b9b5e7f8",
                "c66a821b749e0576e54b89dbac8f71211a508f7916e3d6235900372bed6c6c22",
                "a8bd48117723dee92524c25730f9e08e5d47e78c87d17edb344d4070389d049e"
            ]
        );
    }

    #[test]
    fn test_proof() {
        let tree = get_icons_rgb_circle_tree();
        let result = tree.proof(8);
        assert_eq!(
            result.unwrap(),
            vec![
                "6a10a0b8c1bd3651cba6e5604b31df595e965be137650d296c05afc1084cfe1f",
                "956bf86d100b2f49a8d057ebafa85b8db89a0f19d5627a1226fea1cb3e23d3f3",
                "04284ddea22b003e6098e7dd1a421a565380d11530a35f2e711a8dd2b9b5e7f8",
                "c66a821b749e0576e54b89dbac8f71211a508f7916e3d6235900372bed6c6c22",
                "a8bd48117723dee92524c25730f9e08e5d47e78c87d17edb344d4070389d049e"
            ]
        )
    }

    #[test]
    fn test_piece_data() {
        let tree = get_icons_rgb_circle_tree();

        assert_eq!(tree.piece_data.get(&8).unwrap(),"1wSDXYz+dPEXQP9oAYKE7Tz5ttGgCYkD3ile/OXpP4AAAPTqv+BlsRiHgknDtgQv/orRny7+AhAAgB7a+tKLxbYEp8bkJiY7bdm/L7n35ek/QN/NOQMAGYi+8c17X7AQLf8MUxOjP83+B+jzn71XLs+ZAgQZiKkxO7QCtffz27kjyYu/zP0HGAwtQJCJGA36zFtvWIgWSrWF646n/wACAFBzIfnqL7qTZGiXFC/+uvPpZ0Z/AvTfpW4AmL9yedpaQD5iKpDRoO0q/lMc/an9B2Ag5roBwDpAXuI8wLOnTlqItgSABEd/xsVf97/+xocLMCACAGQoxkkaDdqGz2m8e3YjNXc+1fsPMCDfLQ0As9YD8vLM735jNGjDpdj7Hxd/Gf0JMDAzSwOAUaCQoWdPn3QeoKHi4i+jPwGogxYgyPkPgOefK3YYDdpITyfYouXiL4CBe6AFyG3AkKl4ymw0aLPErkyK7T93P/+imL923QcMMDhagIAFMRo0Wk5ohij+Y1pTam5+OOXDBWgALUBAt9jcaTRoY6Q4oenu+S9d/AUweHNLA8C09YC8xbjJ7a93LMSARTuWi78AqMP8lcsPBACAYvuvj3VnzzM42xK8+CtGf9676KgZQFMsBoA5SwGEnSffNhp0QOJehrikLTV6/wEa4cIDAWBxOwAg2k/iUDD9t83oTwD68b/1S/71VcsBhK0vvZjkGMomi10XF38BUKO5lQKABk3gB8+89Ua3JYX+SPHpf7jj6T9AowMAwA9iNOgOrUACwEaK/08/M/oToDm+WykATFsXYKkYDRo7AdQr1Yu/tP8ANMrMSgEA4CHbXv2F0aB1B4AER3/euzhb3P/6Gx8uQHOsuAPgDACwomdPnzQatCYRrmKnJTV3PtX7D9Ak81cur7gD8J2lAVYS7SnPnjppIWqQYu9/XPxl9CdAc9kBAFYlLqhKdVLNoDz10590R66mRu8/QONcWDEAzF+5bAcAeKxnfvcbo0F76GkXfwEwAMsPAc9aEuBxYjSo8wAbF2uY4mVrdz//opi/dt0HDNAs00v/j83L/p92AViXexdniv+z76VsC7mRf/mnYuj557J4v3FgdcdbbxTX3nnPF38DUh39efPDKR8uQPM8UOMPPS4dAE8WTzu/f/NEVu95+PDLxdZDB3z4G5DieYq757908RdAM808LgDYAYB1iHnnNz86k9V73vn7t7uHWFm7CE8p7hg5/AvQWHOPCwAmAcE6RetDXH6Ui2hfifMArN22V9Mc/RmtgAA0z/yVy48NAHOWCNbv+jvvdaeg5CJGg25/veODX4OYohTrlmIABqCRHno6OfS4dACsTfQ/53Y4dvuvj3Vvs2V1thn9CQ==");
    }

    #[test]
    fn test_piece_data_out_of_bounds() {
        let tree = get_icons_rgb_circle_tree();

        assert_eq!(tree.piece_data.get(&17), None);
    }

    #[test]
    fn test_root_hash() {
        let tree = get_icons_rgb_circle_tree();

        assert_eq!(
            hex::encode(&tree.nodes[1]),
            "9b39e1edb4858f7a3424d5a3d0c4579332640e58e101c29f99314a12329fc60b"
        );
    }

    #[test]
    fn test_0_hash() {
        let tree = get_icons_rgb_circle_tree();
        assert_eq!(hex::encode(&tree.nodes[0]), "");
    }

    #[test]
    #[should_panic]
    fn test_root_hash_empty_tree() {
        let tree = get_empty_tree();

        assert_eq!(
            hex::encode(&tree.nodes[1]),
            "9b39e1edb4858f7a3424d5a3d0c4579332640e58e101c29f99314a12329fc60b"
        );
    }

    #[test]
    fn test_small_tree() {
        let tree = get_single_element_tree();

        assert_eq!(tree.nodes.len(), 4);
        assert_eq!(tree.nodes[0].len(), 0);
    }

    #[test]
    fn test_uncle_small_tree() {
        let tree = get_single_element_tree();
        let uncles = tree.uncle_traversal(1);
        assert_eq!(uncles, None);
    }

    #[test]
    fn test_proof_small_tree() {
        let tree = get_single_element_tree();
        let proof = tree.proof(0);
        assert_eq!(
            proof.unwrap(),
            vec!["0000000000000000000000000000000000000000000000000000000000000000"]
        );
    }

    fn get_icons_rgb_circle_tree() -> MerkleTree {
        MerkleTree::new("test_data/icons_rgb_circle.png")
    }

    fn get_empty_tree() -> MerkleTree {
        MerkleTree::new("test_data/test.txt")
    }

    fn get_single_element_tree() -> MerkleTree {
        MerkleTree::new("test_data/small.txt")
    }
}
