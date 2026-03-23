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
