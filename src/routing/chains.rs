// Rust guideline compliant 2026-02-16
use std::collections::VecDeque;

use super::data::OsmData;

// ---------------------------------------------------------------------------
// Chain building
// ---------------------------------------------------------------------------

/// One OSM way within an ordered chain, possibly traversed in reverse.
#[derive(Debug)]
pub struct Seg {
    /// OSM way ID -- stored for debugging; not consumed by routing logic.
    pub id: u64,
    /// First node in traversal order.
    pub head: u64,
    /// Last node in traversal order.
    pub tail: u64,
    pub reversed: bool,
    /// Index into `OsmData::ways`, used by callers needing the full node list.
    pub way_idx: usize,
}

/// Order `way_indices` (indices into `data.ways`) into one or more connected
/// chains by matching endpoints.
///
/// The greedy algorithm extends each chain at its tail, then its head, until
/// no further connection exists, then starts a new chain with the next
/// unvisited way.  A way is reversed when its last node matches the current
/// chain endpoint rather than its first.
pub fn build_chains(way_indices: &[usize], data: &OsmData) -> Vec<Vec<Seg>> {
    // Use VecDeque so pop_front() is O(1) instead of the O(n) Vec::remove(0).
    let mut remaining: VecDeque<usize> = way_indices.iter().copied().collect();
    let mut chains: Vec<Vec<Seg>> = Vec::new();

    while !remaining.is_empty() {
        let first_idx = remaining
            .pop_front()
            .expect("remaining is non-empty per loop condition");
        let first_way = &data.ways[first_idx];
        let head = first_way.nodes.first().copied().unwrap_or(0);
        let tail = first_way.nodes.last().copied().unwrap_or(0);
        let mut chain = vec![Seg { id: first_way.id, head, tail, reversed: false, way_idx: first_idx }];

        loop {
            let chain_tail = chain.last().expect("chain is non-empty").tail;
            let chain_head = chain.first().expect("chain is non-empty").head;

            // -- Extend at tail: find a way whose head == chain_tail --
            let extend_tail = remaining.iter().position(|&i| {
                let w = &data.ways[i];
                w.nodes.first().copied().unwrap_or(0) == chain_tail
                    || w.nodes.last().copied().unwrap_or(0) == chain_tail
            });
            if let Some(pos) = extend_tail {
                let idx = remaining
                    .remove(pos)
                    .expect("pos came from iter().position(), so it is valid");
                let w = &data.ways[idx];
                let rev = w.nodes.first().copied().unwrap_or(0) != chain_tail;
                let (h, t) = if rev {
                    (w.nodes.last().copied().unwrap_or(0), w.nodes.first().copied().unwrap_or(0))
                } else {
                    (w.nodes.first().copied().unwrap_or(0), w.nodes.last().copied().unwrap_or(0))
                };
                chain.push(Seg { id: w.id, head: h, tail: t, reversed: rev, way_idx: idx });
                continue;
            }

            // -- Extend at head: find a way whose tail == chain_head --
            let extend_head = remaining.iter().position(|&i| {
                let w = &data.ways[i];
                w.nodes.last().copied().unwrap_or(0) == chain_head
                    || w.nodes.first().copied().unwrap_or(0) == chain_head
            });
            if let Some(pos) = extend_head {
                let idx = remaining
                    .remove(pos)
                    .expect("pos came from iter().position(), so it is valid");
                let w = &data.ways[idx];
                let rev = w.nodes.last().copied().unwrap_or(0) != chain_head;
                let (h, t) = if rev {
                    (w.nodes.last().copied().unwrap_or(0), w.nodes.first().copied().unwrap_or(0))
                } else {
                    (w.nodes.first().copied().unwrap_or(0), w.nodes.last().copied().unwrap_or(0))
                };
                chain.insert(0, Seg { id: w.id, head: h, tail: t, reversed: rev, way_idx: idx });
                continue;
            }

            break;
        }

        chains.push(chain);
    }

    chains
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::data::{RawNode, RawWay};
    use std::collections::HashMap;

    // Build a minimal OsmData from a list of (way_id, node_id_list) pairs.
    // The nodes HashMap is left empty because build_chains only uses way.nodes
    // for endpoint matching; coordinate lookup happens in build_polylines.
    fn make_osm(ways: &[(u64, &[u64])]) -> OsmData {
        let raw_ways = ways
            .iter()
            .map(|&(id, nodes)| RawWay {
                id,
                nodes: nodes.to_vec(),
                tags: HashMap::new(),
            })
            .collect();
        let nodes: HashMap<u64, RawNode> = HashMap::new();
        OsmData { ways: raw_ways, nodes }
    }

    #[test]
    fn single_way_produces_one_chain_not_reversed() {
        // A single way [1, 2, 3] must yield exactly one chain of one segment
        // with reversed=false.
        let data = make_osm(&[(10, &[1, 2, 3])]);
        let chains = build_chains(&[0], &data);
        assert_eq!(chains.len(), 1, "one way -> one chain");
        assert_eq!(chains[0].len(), 1, "chain must contain one segment");
        assert!(!chains[0][0].reversed, "single way must not be reversed");
        assert_eq!(chains[0][0].head, 1);
        assert_eq!(chains[0][0].tail, 3);
    }

    #[test]
    fn two_ways_connected_tail_to_head_merge_into_one_chain() {
        // Way A: [1, 2]; Way B: [2, 3].
        // B's head matches A's tail -> they merge into one chain [A, B].
        let data = make_osm(&[(10, &[1, 2]), (11, &[2, 3])]);
        let chains = build_chains(&[0, 1], &data);
        assert_eq!(chains.len(), 1, "two connected ways -> one chain");
        assert_eq!(chains[0].len(), 2, "chain must contain both segments");
        assert!(!chains[0][0].reversed);
        assert!(!chains[0][1].reversed);
        assert_eq!(chains[0][0].head, 1);
        assert_eq!(chains[0][1].tail, 3);
    }

    #[test]
    fn second_way_reversed_when_tail_matches() {
        // Way A: [1, 2]; Way B: [3, 2] (tail == A's tail -> B must be reversed).
        // After reversal B runs 2 -> 3.
        let data = make_osm(&[(10, &[1, 2]), (11, &[3, 2])]);
        let chains = build_chains(&[0, 1], &data);
        assert_eq!(chains.len(), 1, "two connected ways -> one chain");
        assert_eq!(chains[0].len(), 2);
        assert!(!chains[0][0].reversed, "first way must not be reversed");
        assert!(chains[0][1].reversed, "second way must be reversed");
        assert_eq!(chains[0][1].tail, 3, "reversed way tail is the original head");
    }

    #[test]
    fn three_ways_form_single_chain() {
        // Ways: [1,2], [2,3], [3,4] -- all connected head-to-tail.
        let data = make_osm(&[(10, &[1, 2]), (11, &[2, 3]), (12, &[3, 4])]);
        let chains = build_chains(&[0, 1, 2], &data);
        assert_eq!(chains.len(), 1, "three connected ways -> one chain");
        assert_eq!(chains[0].len(), 3);
        assert_eq!(chains[0][0].head, 1);
        assert_eq!(chains[0][2].tail, 4);
    }

    #[test]
    fn two_disconnected_groups_produce_two_chains() {
        // Group A: [1,2] and [2,3]; Group B: [10,11] and [11,12].
        // No connection between groups -> two separate chains.
        let data = make_osm(&[
            (10, &[1, 2]),
            (11, &[2, 3]),
            (20, &[10, 11]),
            (21, &[11, 12]),
        ]);
        let chains = build_chains(&[0, 1, 2, 3], &data);
        assert_eq!(chains.len(), 2, "two disconnected groups -> two chains");
        assert_eq!(chains[0].len() + chains[1].len(), 4, "all four ways distributed");
    }
}