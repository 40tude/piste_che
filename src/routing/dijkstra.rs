// Rust guideline compliant 2026-02-16
use super::graph::Segment;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Sum of haversine distances between consecutive coords, in metres.
pub fn segment_length(seg: &Segment) -> f64 {
    seg.coords
        .windows(2)
        .map(|w| super::data::haversine(w[0][0], w[0][1], w[1][0], w[1][1]))
        .sum()
}

/// Dijkstra shortest path using physical-distance weights, in metres.
///
/// Weights: lift = 50 m fixed, traverse = 10x haversine, everything else = haversine.
/// Stops as soon as any node in `goal_zone` is settled (first-settled =
/// minimum-cost arrival node).  Returns the ordered segment IDs, or `None`
/// when no path exists.
///
/// Returns `Some(vec![])` when `start` is already in `goal_zone`.
pub fn dijkstra(
    start: usize,
    goal_zone: &HashSet<usize>,
    n_nodes: usize,
    segments: &[Segment],
    adj: &HashMap<usize, Vec<usize>>,
    excluded_difficulties: &[&str],
    excluded_lift_types: &[&str],
) -> Option<Vec<usize>> {
    let mut dist = vec![u32::MAX; n_nodes];
    let mut prev: Vec<Option<usize>> = vec![None; n_nodes]; // segment ID that reached each node
    let mut actual_goal: Option<usize> = None;

    dist[start] = 0;
    // Priority queue entries: (cost, node_id)
    let mut heap: BinaryHeap<Reverse<(u32, usize)>> = BinaryHeap::new();
    heap.push(Reverse((0, start)));

    while let Some(Reverse((cost, u))) = heap.pop() {
        if goal_zone.contains(&u) {
            actual_goal = Some(u);
            break;
        }
        if cost > dist[u] {
            continue; // stale entry
        }

        let empty = vec![];
        for &sid in adj.get(&u).unwrap_or(&empty) {
            let seg = &segments[sid];

            // Filter excluded segments.
            let active = match seg.kind.as_str() {
                "piste" => !excluded_difficulties.contains(&seg.difficulty.as_str()),
                "lift" => !excluded_lift_types.contains(&seg.difficulty.as_str()),
                _ => true, // traverse/ski-out/ski-in: always active
            };
            if !active {
                continue;
            }

            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "distance in metres, always positive and < 50 km"
            )]
            let weight: u32 = match seg.kind.as_str() {
                "lift" => 50,
                "traverse" => (segment_length(seg) * 10.0).round() as u32, // heavy penalty
                _ => segment_length(seg).round() as u32,                   // piste, ski-out, ski-in
            };

            let new_cost = cost.saturating_add(weight);
            if new_cost < dist[seg.to] {
                dist[seg.to] = new_cost;
                prev[seg.to] = Some(sid);
                heap.push(Reverse((new_cost, seg.to)));
            }
        }
    }

    let goal = actual_goal?;

    // Reconstruct path by following prev[] backwards.
    let mut path: Vec<usize> = Vec::new();
    let mut cur = goal;
    while let Some(sid) = prev[cur] {
        path.push(sid);
        cur = segments[sid].from;
        if cur == start {
            break;
        }
    }
    path.reverse();
    Some(path)
}

// ---------------------------------------------------------------------------
// Unit tests (T021)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal two-node graph with one segment for testing.
    fn make_segment(id: usize, from: usize, to: usize, kind: &str, difficulty: &str) -> Segment {
        Segment {
            id,
            from,
            to,
            name: format!("seg_{id}"),
            kind: kind.to_string(),
            difficulty: difficulty.to_string(),
            // Two coords 100 m apart (approximately)
            coords: vec![[44.9, 6.5, 1800.0], [44.9009, 6.5, 1750.0]],
            occupancy: None,
            duration_min: None,
        }
    }

    fn make_adj(segments: &[Segment]) -> HashMap<usize, Vec<usize>> {
        let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
        for seg in segments {
            adj.entry(seg.from).or_default().push(seg.id);
        }
        adj
    }

    #[test]
    fn same_start_end_returns_empty_path() {
        // When start is in goal_zone, Dijkstra returns Some(vec![]) without traversing.
        let segments: Vec<Segment> = vec![make_segment(0, 0, 1, "piste", "easy")];
        let adj = make_adj(&segments);
        let mut goal = HashSet::new();
        goal.insert(0); // start == goal

        let result = dijkstra(0, &goal, 2, &segments, &adj, &[], &[]);
        assert_eq!(result, Some(vec![]), "same start/end must return empty path");
    }

    #[test]
    fn disconnected_nodes_returns_none() {
        // No segments connecting node 0 to node 1 -> no path.
        let segments: Vec<Segment> = vec![];
        let adj = HashMap::new();
        let mut goal = HashSet::new();
        goal.insert(1);

        let result = dijkstra(0, &goal, 2, &segments, &adj, &[], &[]);
        assert_eq!(result, None, "disconnected graph must return None");
    }

    #[test]
    fn simple_path_found() {
        // Single segment 0->1; goal = {1}; should return [0].
        let segments = vec![make_segment(0, 0, 1, "piste", "easy")];
        let adj = make_adj(&segments);
        let mut goal = HashSet::new();
        goal.insert(1);

        let result = dijkstra(0, &goal, 2, &segments, &adj, &[], &[]);
        assert_eq!(result, Some(vec![0]), "simple path must return segment 0");
    }

    #[test]
    fn excluded_difficulty_blocks_path() {
        // Segment 0->1 has difficulty "advanced"; excluding it leaves no path.
        let segments = vec![make_segment(0, 0, 1, "piste", "advanced")];
        let adj = make_adj(&segments);
        let mut goal = HashSet::new();
        goal.insert(1);

        let result = dijkstra(0, &goal, 2, &segments, &adj, &["advanced"], &[]);
        assert_eq!(result, None, "excluding difficulty must block path");
    }

    #[test]
    fn excluded_lift_type_blocks_path() {
        // Segment 0->1 is a chair_lift; excluding it leaves no path.
        let segments = vec![make_segment(0, 0, 1, "lift", "chair_lift")];
        let adj = make_adj(&segments);
        let mut goal = HashSet::new();
        goal.insert(1);

        let result = dijkstra(0, &goal, 2, &segments, &adj, &[], &["chair_lift"]);
        assert_eq!(result, None, "excluding lift type must block path");
    }

    #[test]
    fn excluded_difficulty_path_uses_alternate() {
        // Segments: 0->1 (advanced, blocked), 0->2->1 (easy, allowed).
        // Expected: path via 2 using segments 1 and 2.
        let segments = vec![
            make_segment(0, 0, 1, "piste", "advanced"), // blocked
            make_segment(1, 0, 2, "piste", "easy"),     // node 0->2
            make_segment(2, 2, 1, "piste", "easy"),     // node 2->1
        ];
        let adj = make_adj(&segments);
        let mut goal = HashSet::new();
        goal.insert(1);

        let result = dijkstra(0, &goal, 3, &segments, &adj, &["advanced"], &[]);
        assert!(result.is_some(), "alternate path must be found");
        let path = result.unwrap();
        assert!(!path.contains(&0), "blocked segment must not appear in path");
    }
}
