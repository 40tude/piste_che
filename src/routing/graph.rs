// Rust guideline compliant 2026-02-16
use super::chains::build_chains;
use super::data::{OsmData, haversine};
use std::collections::{BTreeMap, HashMap, HashSet};

// ---------------------------------------------------------------------------
// Tuning constants (sized for Serre Chevalier)
// ---------------------------------------------------------------------------

/// This is the identity threshold between two points throughout the pipeline
/// Two candidate coords within this distance are merged into one node.
/// Smaller than GPS noise on OSM data; avoids duplicate junction nodes.
const CLUSTER_RADIUS: f64 = 25.0;

/// Detection radius for mid-track junctions with existing endpoints
const SPLIT_RADIUS: f64 = 300.0;

/// Max altitude difference between a split candidate and the nearby endpoint.
/// Prevents linking upper-mountain junctions to valley-level nodes.
const SPLIT_MAX_ALT: f64 = 100.0;

/// Max horizontal distance for a synthetic flat traverse edge.
/// Sized for co-located element gap-bridging only (e.g. piste end to
/// adjacent lift base).  100 m prevents traverse chains that span
/// mountain sections and let Dijkstra skip piste descents entirely.
const TRAVERSE_RADIUS: f64 = 100.0;

/// Max altitude difference allowed for a traverse edge.
/// 30 m keeps traverses on flat terrain; larger values let Dijkstra treat
/// traverse edges as free descents and bypass proper piste segments.
const TRAVERSE_MAX_ALT: f64 = 30.0;

/// Max distance between two piste interior points to detect a crossing.
/// When two pistes pass within this distance at similar altitude, both get
/// a split node, enabling mid-run transitions between pistes.
/// 50 m is generous for GPS noise / wide pistes while avoiding false
/// positives between parallel pistes separated by terrain.
const CROSSING_RADIUS: f64 = 50.0;

/// Max altitude difference for piste crossing detection.
/// Tighter than `TRAVERSE_MAX_ALT`: two pistes must be at nearly the
/// same level to be considered a real crossing (not a vertical overlap).
const CROSSING_MAX_ALT: f64 = 5.0;

/// Radius for directed lift-exit to piste "ski-out" edges.
/// Must be >= `SPLIT_RADIUS` (300 m) so every split node triggered by a
/// lift exit is reachable from that exit via a ski-out edge.
const SKI_OUT_RADIUS: f64 = 350.0;

/// Max descent (lift-exit elevation minus target elevation) for a ski-out edge.
/// Prevents connecting to nodes far down the mountain; GPS noise allows a
/// small negative value (target slightly above exit).
const SKI_OUT_MAX_ALT: f64 = 30.0;

/// Horizontal radius (metres) for ski-in edges (Step 6c) and the arrival zone.
///
/// - Step 6c: piste nodes within this radius of a lift base receive a directed
///   ski-in edge toward that base, bridging approach gaps > `TRAVERSE_RADIUS`.
/// - `arrival_zone`: any node within this radius of the destination counts as arrived.
///
/// Must be >= `SPLIT_RADIUS` (300 m) by the same argument as `SKI_OUT_RADIUS`.
pub const SKI_IN_RADIUS: f64 = 350.0;

/// Max altitude gain (metres) from a source node to a lift base for a ski-in edge,
/// and max altitude difference for the arrival zone.
///
/// 30 m prevents connecting to lift bases that are significantly higher than
/// the skier's current position; GPS noise allows 10 m in the other direction
/// (see Step 6c altitude check).
pub const SKI_IN_MAX_ALT: f64 = 30.0;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A unique position in the ski domain graph.
#[derive(Debug)]
pub struct Node {
    pub id: usize,
    /// `[latitude, longitude, elevation_m]`
    pub coord: [f64; 3],
}

/// A directed arc connecting two nodes.
///
/// Corresponds to a segment of a named piste, a lift, or a synthetic traverse.
#[derive(Debug)]
pub struct Segment {
    pub id: usize,
    pub from: usize,
    pub to: usize,
    pub name: String,
    pub kind: String,
    pub difficulty: String,
    pub coords: Vec<[f64; 3]>,
}

/// A named ski domain element with its graph entry and exit nodes.
#[derive(Debug)]
pub struct RouteElement {
    pub name: String,
    pub kind: String,
    pub difficulty: String,
    /// Node where a skier enters this element (top of piste; base of lift).
    pub start_node: usize,
    /// Node where a skier exits this element (bottom of piste; top of lift).
    pub end_node: usize,
}

// ---------------------------------------------------------------------------
// Internal: raw polyline
// ---------------------------------------------------------------------------

struct Polyline {
    group_key: String,
    kind: String,
    difficulty: String,
    /// Ordered sequence of `[lat, lon, ele_m]` points.
    coords: Vec<[f64; 3]>,
}

/// Group OSM ways by `group_key`, chain them, and flatten to coordinate lists.
fn build_polylines(data: &OsmData) -> Vec<Polyline> {
    let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, way) in data.ways.iter().enumerate() {
        if way.is_closed_polygon() || way.element_kind() == "?" {
            continue;
        }
        if let Some(key) = way.group_key() {
            groups.entry(key).or_default().push(i);
        }
    }

    let mut polylines = Vec::new();
    for (group_key, indices) in &groups {
        let first_way = &data.ways[indices[0]];
        let kind = first_way.element_kind().to_string();
        let difficulty = first_way.difficulty().to_string();

        for chain in build_chains(indices, data) {
            let mut coords: Vec<[f64; 3]> = Vec::new();
            for seg in &chain {
                let way = &data.ways[seg.way_idx];
                // Iterate nodes in traversal order; skip duplicate junction node.
                let skip = usize::from(!coords.is_empty());
                let node_ids: Vec<u64> = if seg.reversed {
                    way.nodes.iter().copied().rev().collect()
                } else {
                    way.nodes.clone()
                };
                for nid in node_ids.into_iter().skip(skip) {
                    if let Some(node) = data.nodes.get(&nid) {
                        coords.push([node.lat, node.lon, f64::from(node.ele.unwrap_or(0.0))]);
                    }
                }
            }
            // Normalize direction using elevation so all segments are correctly
            // directed: lifts run base->summit, pistes run summit->base.
            // Skip normalization when both endpoints lack elevation data.
            if let (Some(&first), Some(&last)) = (coords.first(), coords.last()) {
                let both_missing = first[2] == 0.0 && last[2] == 0.0;
                if !both_missing {
                    let needs_reverse = if kind == "lift" {
                        first[2] > last[2] // stored top->base; flip to base->top
                    } else {
                        first[2] < last[2] // stored base->top; flip to top->base
                    };
                    if needs_reverse {
                        coords.reverse();
                    }
                }
            }

            if coords.len() >= 2 {
                polylines.push(Polyline {
                    group_key: group_key.clone(),
                    kind: kind.clone(),
                    difficulty: difficulty.clone(),
                    coords,
                });
            }
        }
    }
    polylines
}

/// Return the node ID of the node whose lat/lon is closest to `coord`.
fn closest_node_id(coord: &[f64; 3], nodes: &[Node]) -> Option<usize> {
    nodes
        .iter()
        .min_by(|a, b| {
            let da = haversine(coord[0], coord[1], a.coord[0], a.coord[1]);
            let db = haversine(coord[0], coord[1], b.coord[0], b.coord[1]);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|n| n.id)
}

/// Determine entry and exit node IDs for a polyline based on elevation.
///
/// Piste: entry = higher end (top); lift: entry = lower end (valley station).
fn entry_exit(pl: &Polyline, nodes: &[Node]) -> Option<(usize, usize)> {
    let &head = pl.coords.first()?;
    let &tail = pl.coords.last()?;
    let head_node = closest_node_id(&head, nodes)?;
    let tail_node = closest_node_id(&tail, nodes)?;

    let (start, end) = if pl.kind == "piste" {
        // Plan: "piste: start=head/top, end=tail/bottom" -- use elevation.
        if head[2] >= tail[2] {
            (head_node, tail_node)
        } else {
            (tail_node, head_node)
        }
    } else {
        // Plan: "lift: inverted" -- boarding is always the lower end.
        if head[2] <= tail[2] {
            (head_node, tail_node)
        } else {
            (tail_node, head_node)
        }
    };
    Some((start, end))
}

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Build the ski domain graph from raw OSM data.
///
/// Returns `(nodes, segments, route_elements)` where:
/// - `nodes` are unique positions clustered from endpoints and split points;
/// - `segments` are directed arcs (piste, lift, or traverse);
/// - `route_elements` map each named element to its entry/exit nodes.
#[expect(
    clippy::too_many_lines,
    reason = "7-step pipeline; splitting would obscure the algorithm flow"
)]
pub fn build_graph(data: &OsmData) -> (Vec<Node>, Vec<Segment>, Vec<RouteElement>) {
    // --- Step 1: Build Polylines
    let polylines = build_polylines(data);

    // --- Step 2: Collect endpoint candidates ---
    let mut candidates: Vec<[f64; 3]> = Vec::new();
    for pl in &polylines {
        if let (Some(&h), Some(&t)) = (pl.coords.first(), pl.coords.last()) {
            candidates.push(h);
            candidates.push(t);
        }
    }

    // --- Step 3: Mid-piste split detection ---
    // Pass A: any interior piste coord near an existing endpoint candidate.
    let mut raw_splits: Vec<[f64; 3]> = Vec::new();
    for pl in &polylines {
        if pl.kind != "piste" {
            continue;
        }
        let n = pl.coords.len();
        for i in 1..n.saturating_sub(1) {
            let pt = pl.coords[i];
            let is_split = candidates.iter().any(|&c| {
                let d = haversine(pt[0], pt[1], c[0], c[1]);
                let de = (pt[2] - c[2]).abs();
                // Require minimum separation to avoid matching the polyline's own endpoints.
                d > 1.0 && d < SPLIT_RADIUS && de < SPLIT_MAX_ALT
            });
            if is_split {
                raw_splits.push(pt);
            }
        }
    }

    // Pass C: piste-to-piste interior crossing detection.
    // When two interior points from different pistes are close (same altitude
    // band), BOTH are added as split candidates.  After clustering, they merge
    // into a shared node enabling mid-run transitions between pistes.
    let piste_indices: Vec<usize> = polylines
        .iter()
        .enumerate()
        .filter(|(_, pl)| pl.kind == "piste")
        .map(|(i, _)| i)
        .collect();
    for (ai, &idx_a) in piste_indices.iter().enumerate() {
        let coords_a = &polylines[idx_a].coords;
        let n_a = coords_a.len();
        for &idx_b in &piste_indices[ai + 1..] {
            let coords_b = &polylines[idx_b].coords;
            for pt_a in &coords_a[1..n_a.saturating_sub(1)] {
                for pt_b in &coords_b[1..coords_b.len().saturating_sub(1)] {
                    let d = haversine(pt_a[0], pt_a[1], pt_b[0], pt_b[1]);
                    let de = (pt_a[2] - pt_b[2]).abs();
                    if d > 1.0 && d < CROSSING_RADIUS && de < CROSSING_MAX_ALT {
                        raw_splits.push(*pt_a);
                        raw_splits.push(*pt_b);
                        break; // one match per pt_a is enough
                    }
                }
            }
        }
    }

    // Pass B: merge nearby raw splits to avoid duplicate split nodes.
    let mut merged_splits: Vec<[f64; 3]> = Vec::new();
    for sp in raw_splits {
        let already = merged_splits
            .iter()
            .any(|&m| haversine(sp[0], sp[1], m[0], m[1]) < CLUSTER_RADIUS);
        if !already {
            merged_splits.push(sp);
        }
    }
    candidates.extend(merged_splits);

    // --- Step 4: Cluster all candidates into unique nodes ---
    let mut nodes: Vec<Node> = Vec::new();
    for coord in candidates {
        let already = nodes
            .iter()
            .any(|n| haversine(coord[0], coord[1], n.coord[0], n.coord[1]) < CLUSTER_RADIUS);
        if !already {
            let id = nodes.len();
            nodes.push(Node { id, coord });
        }
    }

    // --- Step 5: Build directed segments between consecutive boundary nodes ---
    let mut segments: Vec<Segment> = Vec::new();
    for pl in &polylines {
        // Locate every node that sits on this polyline.
        let mut boundaries: Vec<(usize, usize)> = Vec::new(); // (coord_idx, node_id)
        for (ci, coord) in pl.coords.iter().enumerate() {
            for node in &nodes {
                if haversine(coord[0], coord[1], node.coord[0], node.coord[1]) < CLUSTER_RADIUS {
                    boundaries.push((ci, node.id));
                    break;
                }
            }
        }

        // Sort by position, then collapse adjacent duplicates.
        boundaries.sort_by_key(|&(ci, _)| ci);
        boundaries.dedup_by_key(|b| b.1);

        // One segment per consecutive node pair.
        for w in boundaries.windows(2) {
            let (ci_from, nid_from) = w[0];
            let (ci_to, nid_to) = w[1];
            if nid_from == nid_to {
                continue;
            }
            let seg_coords = pl.coords[ci_from..=ci_to].to_vec();
            let id = segments.len();
            segments.push(Segment {
                id,
                from: nid_from,
                to: nid_to,
                name: pl.group_key.clone(),
                kind: pl.kind.clone(),
                difficulty: pl.difficulty.clone(),
                coords: seg_coords,
            });
        }
    }

    // --- Step 6: Synthetic bidirectional traverse edges ---
    //
    // Any two nodes within TRAVERSE_RADIUS with a small altitude difference
    // get a pair of synthetic bidirectional edges (kind "traverse") to model
    // short flat transitions between run exits and adjacent element bases.
    //
    // Kind "traverse" carries a 10x distance penalty in Dijkstra so that
    // chaining many traverse hops is never cheaper than taking a real piste.
    let n_nodes = nodes.len();
    for i in 0..n_nodes {
        for j in (i + 1)..n_nodes {
            let d = haversine(
                nodes[i].coord[0],
                nodes[i].coord[1],
                nodes[j].coord[0],
                nodes[j].coord[1],
            );
            let de = (nodes[i].coord[2] - nodes[j].coord[2]).abs();
            if d < TRAVERSE_RADIUS && de < TRAVERSE_MAX_ALT {
                let id_fwd = segments.len();
                segments.push(Segment {
                    id: id_fwd,
                    from: i,
                    to: j,
                    name: "traverse".to_string(),
                    kind: "traverse".to_string(),
                    difficulty: "-".to_string(),
                    coords: vec![nodes[i].coord, nodes[j].coord],
                });
                let id_rev = segments.len();
                segments.push(Segment {
                    id: id_rev,
                    from: j,
                    to: i,
                    name: "traverse".to_string(),
                    kind: "traverse".to_string(),
                    difficulty: "-".to_string(),
                    coords: vec![nodes[j].coord, nodes[i].coord],
                });
            }
        }
    }

    // --- Step 6b: Directed ski-out edges from each lift exit ---
    //
    // Bridges the gap between a lift summit and the nearest accessible piste.
    // Directed only (lift-exit -> piste node); not bidirectional, so Dijkstra
    // cannot use them as reverse shortcuts.
    //
    // Excluded targets: other lift-exit nodes (avoids lift-to-lift shortcuts)
    // and lift-base nodes (skier cannot ski-out directly to a next lift base).
    {
        let lift_base_ids: HashSet<usize> = segments
            .iter()
            .filter(|s| s.kind == "lift")
            .map(|s| s.from) // after normalization: from = base
            .collect();
        let lift_exit_ids: HashSet<usize> = segments
            .iter()
            .filter(|s| s.kind == "lift")
            .map(|s| s.to) // after normalization: to = summit
            .collect();

        let exit_ids: Vec<usize> = lift_exit_ids.iter().copied().collect();
        for exit_id in exit_ids {
            let exit = nodes[exit_id].coord;
            for node in &nodes {
                let node_id = node.id;
                // Skip lift bases and other lift exits.
                if lift_base_ids.contains(&node_id) || lift_exit_ids.contains(&node_id) {
                    continue;
                }
                if node_id == exit_id {
                    continue;
                }
                let target = node.coord;
                let d = haversine(exit[0], exit[1], target[0], target[1]);
                let descent = exit[2] - target[2]; // positive = target is below exit
                if d > 1.0
                    && d < SKI_OUT_RADIUS
                    && descent > -10.0 // allow <=10 m uphill (GPS noise)
                    && descent < SKI_OUT_MAX_ALT
                {
                    let id = segments.len();
                    segments.push(Segment {
                        id,
                        from: exit_id,
                        to: node_id,
                        name: "ski-out".to_string(),
                        kind: "ski-out".to_string(),
                        difficulty: "-".to_string(),
                        coords: vec![exit, target],
                    });
                }
            }
        }
    }

    // --- Step 6c: Directed ski-in edges toward each lift base ---
    //
    // Mirrors Step 6b in reverse: bridges the approach gap between the end of a
    // piste and the nearest lift boarding point when the distance exceeds
    // TRAVERSE_RADIUS.  Directed only (piste node -> lift base); Dijkstra cannot
    // use them as descent shortcuts.
    //
    // Excluded sources: lift-exit and lift-base nodes to prevent lift-to-lift
    // and base-to-base shortcuts.
    {
        let lift_base_ids: HashSet<usize> = segments
            .iter()
            .filter(|s| s.kind == "lift")
            .map(|s| s.from) // after normalization: from = base
            .collect();
        let lift_exit_ids: HashSet<usize> = segments
            .iter()
            .filter(|s| s.kind == "lift")
            .map(|s| s.to) // after normalization: to = summit
            .collect();

        let base_ids: Vec<usize> = lift_base_ids.iter().copied().collect();
        for base_id in base_ids {
            let base = nodes[base_id].coord;
            for node in &nodes {
                let node_id = node.id;
                // Skip other lift bases and lift exits.
                if lift_base_ids.contains(&node_id) || lift_exit_ids.contains(&node_id) {
                    continue;
                }
                if node_id == base_id {
                    continue;
                }
                let source = node.coord;
                let d = haversine(source[0], source[1], base[0], base[1]);
                let ascent = base[2] - source[2]; // positive = base is above source
                if d > 1.0
                    && d < SKI_IN_RADIUS
                    && ascent > -10.0 // allow <=10 m downhill (GPS noise)
                    && ascent < SKI_IN_MAX_ALT
                {
                    let id = segments.len();
                    segments.push(Segment {
                        id,
                        from: node_id,
                        to: base_id,
                        name: "ski-in".to_string(),
                        kind: "ski-in".to_string(),
                        difficulty: "-".to_string(),
                        coords: vec![source, base],
                    });
                }
            }
        }
    }

    // --- Step 7: Build one RouteElement per named element ---
    let mut seen: BTreeMap<String, RouteElement> = BTreeMap::new();
    for pl in &polylines {
        if seen.contains_key(&pl.group_key) {
            continue;
        }
        let Some((start_node, end_node)) = entry_exit(pl, &nodes) else {
            continue;
        };
        seen.insert(
            pl.group_key.clone(),
            RouteElement {
                name: pl.group_key.clone(),
                kind: pl.kind.clone(),
                difficulty: pl.difficulty.clone(),
                start_node,
                end_node,
            },
        );
    }
    let route_elements: Vec<RouteElement> = seen.into_values().collect();

    (nodes, segments, route_elements)
}

/// Build an adjacency list mapping each node ID to the segment IDs departing from it.
pub fn adjacency_from_segments(segments: &[Segment]) -> HashMap<usize, Vec<usize>> {
    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for seg in segments {
        adj.entry(seg.from).or_default().push(seg.id);
    }
    adj
}

/// Return the set of node IDs within the arrival zone of `goal_node`.
///
/// Any node within [`SKI_IN_RADIUS`] metres horizontal and [`SKI_IN_MAX_ALT`]
/// metres vertical of `goal_node` is included, as well as `goal_node` itself.
/// Passing this set as the Dijkstra goal lets the search stop as soon as the
/// skier reaches any node in the lift-base vicinity, preventing short connector
/// pistes from appearing as extra itinerary steps.
pub fn arrival_zone(goal_node: usize, nodes: &[Node]) -> HashSet<usize> {
    let gc = nodes[goal_node].coord;
    nodes
        .iter()
        .filter(|n| {
            let d = haversine(gc[0], gc[1], n.coord[0], n.coord[1]);
            let de = (gc[2] - n.coord[2]).abs();
            d < SKI_IN_RADIUS && de < SKI_IN_MAX_ALT
        })
        .map(|n| n.id)
        .collect()
}
