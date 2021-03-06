// Copyright 2021 Stichting Organism
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// Copyright (c) 2019 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

use crate::hash::H256;

// fast_merkle_root treats the provided slice of hashes as leaves of a merkle tree
// and returns the resulting merkle root.
//
// A merkle tree is a tree in which every non-leaf node is the hash of its
// children nodes.  A diagram depicting how this works for Organism transactions
// where h(x) is a blake256 hash follows:
//
//	         root = h1234 = h(h12 + h34)
//	        /                           \
//	  h12 = h(h1 + h2)            h34 = h(h3 + h4)
//	   /            \              /            \
//	h1 = h(tx1)  h2 = h(tx2)    h3 = h(tx3)  h4 = h(tx4)
//
// The number of inputs is not always a power of two which results in a
// balanced tree structure as above.  In that case, parent nodes with no
// children are also zero and parent nodes with only a single left node
// are calculated by concatenating the left node with itself before hashing.
pub fn fast_merkle_root(mut leaves: Vec<H256>) -> H256 {
    // All zero.
    if leaves.len() == 0 {
        return H256::zero();
    }

    if leaves.len() < 2 {
        return leaves[0];
    }
    // The following algorithm works by replacing the leftmost entries in the
    // slice with the concatenations of each subsequent set of 2 hashes and
    // shrinking the slice by half to account for the fact that each level of
    // the tree is half the size of the previous one.  In the case a level is
    // unbalanced (there is no final right child), the final node is duplicated
    // so it ultimately is concatenated with itself.
    //
    // For example, the following illustrates calculating a tree with 5 leaves:
    //
    // [0 1 2 3 4]                              (5 entries)
    // 1st iteration: [h(0||1) h(2||3) h(4||4)] (3 entries)
    // 2nd iteration: [h(h01||h23) h(h44||h44)] (2 entries)
    // 3rd iteration: [h(h0123||h4444)]         (1 entry)
    while leaves.len() > 1 {
        // When there is no right child, the parent is generated by hashing the
        // concatenation of the left child with itself.
        if leaves.len() & 1 != 0 {
            leaves.push(leaves[leaves.len() - 1]);
        }

        // Set the parent node to the hash of the concatenation of the left and
        // right children.
        let mut i = 0;
        while i < leaves.len() / 2 {
            let left = leaves[i * 2].hash_with(leaves[i * 2 + 1]);
            leaves[i] = left;
            i += 1;
        }
        let drain_start = leaves.len() / 2;
        leaves.split_off(drain_start);
    }

    leaves[0]
}

#[test]
fn test_to_merkle_fast_short() {
    let _inputs = vec![
        H256::from_hex("5e574591d900f7f9abb8f8eb31cc9330247d27ba293ad79c348d602ece717b8b").unwrap(),
        H256::from_hex("b3b70fe08c2da744c9559d533e8db35b3bfefba1b0f1c7b31e7d9d523c00a426").unwrap(),
        H256::from_hex("dd3058a7fc691ff4dee0a8cd6030f404ffda7e7aee88aff3985f7b2bbe4792f7").unwrap(),
        H256::from_hex("5e574591d900f7f9abb8f8eb31cc9330247d27ba293ad79c348d602ece717b8b").unwrap(),
        H256::from_hex("b3b70fe08c2da744c9559d533e8db35b3bfefba1b0f1c7b31e7d9d523c00a426").unwrap(),
        H256::from_hex("dd3058a7fc691ff4dee0a8cd6030f404ffda7e7aee88aff3985f7b2bbe4792f7").unwrap(),
        H256::from_hex("5e574591d900f7f9abb8f8eb31cc9330247d27ba293ad79c348d602ece717b8b").unwrap(),
        H256::from_hex("b3b70fe08c2da744c9559d533e8db35b3bfefba1b0f1c7b31e7d9d523c00a426").unwrap(),
        H256::from_hex("dd3058a7fc691ff4dee0a8cd6030f404ffda7e7aee88aff3985f7b2bbe4792f7").unwrap(),
    ];

    //assert_eq!(fast_merkle_root(&mut inputs),  H256::from_hex("a144c719391569aa20bf612bf5588bce71cd397574cb6c060e0bac100f6e5805").unwrap());
}

#[test]
fn test_to_merkle_fast_zero() {
    assert_eq!(fast_merkle_root(vec![H256::zero()]), H256::zero());
}
