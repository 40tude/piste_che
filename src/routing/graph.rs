// Rust guideline compliant 2026-03-23
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
/// 5 m keeps traverses on flat terrain; larger values let Dijkstra treat
/// traverse edges as free descents and bypass proper piste segments.
const TRAVERSE_MAX_ALT: f64 = 5.0;

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
///
/// Bridges the immediate gap between a lift summit station and the piste
/// nodes that depart from that summit.  Kept tight (100 m) to prevent
/// Dijkstra from treating a ski-out edge as a free long-distance shortcut.
/// Split nodes further than 100 m from the lift exit are still reachable
/// via regular piste segments or traverse edges.
const SKI_OUT_RADIUS: f64 = 100.0;

/// Max descent (lift-exit elevation minus target elevation) for a ski-out edge.
/// Prevents connecting to nodes far down the mountain; GPS noise allows a
/// small negative value (target slightly above exit).
const SKI_OUT_MAX_ALT: f64 = 10.0;

/// Horizontal radius (metres) for ski-in edges (Step 6c) and the arrival zone.
///
/// - Step 6c: piste nodes within this radius of a lift base receive a directed
///   ski-in edge toward that base, bridging approach gaps > `TRAVERSE_RADIUS`.
/// - `arrival_zone`: any node within this radius of the destination counts as arrived.
///
/// Kept tight (100 m) so that ski-in edges only bridge the final approach
/// to the boarding station, not arbitrary cross-mountain shortcuts.
/// Piste nodes further away reach the lift base via regular piste segments.
pub const SKI_IN_RADIUS: f64 = 100.0;

/// Max altitude gain (metres) from a source node to a lift base for a ski-in edge,
/// and max altitude difference for the arrival zone.
///
/// 30 m prevents connecting to lift bases that are significantly higher than
/// the skier's current position; GPS noise allows 10 m in the other direction
/// (see Step 6c altitude check).
pub const SKI_IN_MAX_ALT: f64 = 10.0;

/// Max descent (metres) from a piste source node to a lift base for a ski-in edge.
///
/// A lift base station can sit slightly below the end of the approach piste
/// (the skier glides down a short ramp to board).  25 m covers real terrain
/// gaps seen in Serre Chevalier data (e.g. Vauban end at 1219 m -> Prorel 1
/// base at 1204 m: 15.5 m descent) while staying tighter than SPLIT_MAX_ALT
/// so that distant downhill nodes are never bridged by a ski-in edge.
pub const SKI_IN_MAX_DESCENT: f64 = 25.0;

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
    /// Number of seats per cabin/chair (aerialway:occupancy tag), lifts only.
    pub occupancy: Option<u32>,
    /// Ride duration in minutes (aerialway:duration tag), lifts only.
    pub duration_min: Option<u32>,
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
    /// Seats per cabin/chair (aerialway:occupancy), lifts only.
    occupancy: Option<u32>,
    /// Ride duration in minutes (aerialway:duration), lifts only.
    duration_min: Option<u32>,
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
        let occupancy = first_way.occupancy();
        let duration_min = first_way.duration_min();

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
            //
            // Elevation sentinel: nodes without elevation are stored as 0.0
            // (the default when `ele` is absent in the JSON, see OsmData::load).
            // Treating 0.0 as "missing" is valid for Serre Chevalier (all
            // elements are above 1000 m), but would misclassify sea-level
            // elements if this code were reused for a coastal resort.
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
                    occupancy,
                    duration_min,
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

        if pl.kind == "lift" {
            // Lifts must never be split mid-way: boarding is only possible at
            // the base station.  Restrict to first and last coord so Dijkstra
            // cannot enter a lift via a traverse edge to an interior node.
            let n = pl.coords.len();
            for &ci in &[0, n - 1] {
                let coord = pl.coords[ci];
                for node in &nodes {
                    if haversine(coord[0], coord[1], node.coord[0], node.coord[1]) < CLUSTER_RADIUS
                    {
                        if !boundaries.iter().any(|&(_, nid)| nid == node.id) {
                            boundaries.push((ci, node.id));
                        }
                        break;
                    }
                }
            }
        } else {
            for (ci, coord) in pl.coords.iter().enumerate() {
                for node in &nodes {
                    if haversine(coord[0], coord[1], node.coord[0], node.coord[1]) < CLUSTER_RADIUS
                    {
                        boundaries.push((ci, node.id));
                        break;
                    }
                }
            }
        }

        // Sort by coord position, then deduplicate by node ID keeping the
        // first (lowest-ci) occurrence of each node.
        //
        // `dedup_by_key` only removes *consecutive* duplicates; if the same
        // node appears at ci=10 and ci=25 (non-adjacent after sort) both
        // entries survive, creating a topology loop through that node.
        // The retain+HashSet approach removes ALL duplicate node IDs globally,
        // not just adjacent ones, regardless of how far apart they are.
        boundaries.sort_by_key(|&(ci, _)| ci);
        {
            let mut seen = HashSet::new();
            boundaries.retain(|&(_, nid)| seen.insert(nid));
        }

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
                occupancy: pl.occupancy,
                duration_min: pl.duration_min,
            });
        }
    }

    // Snapshot lift node sets from the directed piste/lift segments built in
    // Step 5.  Steps 6a-6c each need these sets; computing them after Step 6a
    // would include traverse edges in the filter (harmless today, but a hidden
    // ordering dependency).  Snapshotting here makes the dependency explicit.
    let lift_base_ids_snap: HashSet<usize> = segments
        .iter()
        .filter(|s| s.kind == "lift")
        .map(|s| s.from) // after normalization: from = base station
        .collect();
    let lift_exit_ids_snap: HashSet<usize> = segments
        .iter()
        .filter(|s| s.kind == "lift")
        .map(|s| s.to) // after normalization: to = summit station
        .collect();

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
                    occupancy: None,
                    duration_min: None,
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
                    occupancy: None,
                    duration_min: None,
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
    // One edge per reachable piste: among all nodes of the same named piste
    // within the radius, only the closest one receives a ski-out edge.
    // Connecting every node would let Dijkstra skip the start of a piste by
    // entering further down, which is physically unrealistic.
    //
    // Excluded targets: other lift-exit nodes (avoids lift-to-lift shortcuts)
    // and lift-base nodes (skier cannot ski-out directly to a next lift base).
    {
        let lift_base_ids = &lift_base_ids_snap;
        let lift_exit_ids = &lift_exit_ids_snap;

        // Map each node to the piste names that use it (from/to of piste segments).
        let mut node_piste_names: HashMap<usize, Vec<String>> = HashMap::new();
        for seg in &segments {
            if seg.kind == "piste" {
                node_piste_names
                    .entry(seg.from)
                    .or_default()
                    .push(seg.name.clone());
                node_piste_names
                    .entry(seg.to)
                    .or_default()
                    .push(seg.name.clone());
            }
        }

        let exit_ids: Vec<usize> = lift_exit_ids.iter().copied().collect();
        for exit_id in exit_ids {
            let exit = nodes[exit_id].coord;

            // For each reachable piste name, keep only the closest valid node.
            let mut closest_by_piste: HashMap<String, (f64, usize)> = HashMap::new();
            for node in &nodes {
                let node_id = node.id;
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
                    if let Some(piste_names) = node_piste_names.get(&node_id) {
                        for piste_name in piste_names {
                            let entry = closest_by_piste
                                .entry(piste_name.clone())
                                .or_insert((d, node_id));
                            if d < entry.0 {
                                *entry = (d, node_id);
                            }
                        }
                    }
                }
            }

            // One ski-out edge per reachable piste, to the closest entry node.
            for (_, (_, target_id)) in closest_by_piste {
                let target = nodes[target_id].coord;
                let id = segments.len();
                segments.push(Segment {
                    id,
                    from: exit_id,
                    to: target_id,
                    name: "ski-out".to_string(),
                    kind: "ski-out".to_string(),
                    difficulty: "-".to_string(),
                    coords: vec![exit, target],
                    occupancy: None,
                    duration_min: None,
                });
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
    // One edge per reachable piste: among all nodes of the same named piste
    // within the radius, only the closest one receives a ski-in edge.
    // Connecting every node would let Dijkstra skip the end of a piste by
    // branching off to the lift base earlier, which is physically unrealistic.
    //
    // Excluded sources: lift-exit and lift-base nodes to prevent lift-to-lift
    // and base-to-base shortcuts.
    {
        let lift_base_ids = &lift_base_ids_snap;
        let lift_exit_ids = &lift_exit_ids_snap;

        // Map each node to the piste names that use it (from/to of piste segments).
        let mut node_piste_names: HashMap<usize, Vec<String>> = HashMap::new();
        for seg in &segments {
            if seg.kind == "piste" {
                node_piste_names
                    .entry(seg.from)
                    .or_default()
                    .push(seg.name.clone());
                node_piste_names
                    .entry(seg.to)
                    .or_default()
                    .push(seg.name.clone());
            }
        }

        let base_ids: Vec<usize> = lift_base_ids.iter().copied().collect();
        for base_id in base_ids {
            let base = nodes[base_id].coord;

            // For each reachable piste name, keep only the closest valid node.
            let mut closest_by_piste: HashMap<String, (f64, usize)> = HashMap::new();
            for node in &nodes {
                let node_id = node.id;
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
                    && ascent > -SKI_IN_MAX_DESCENT // allow short descent to boarding area
                    && ascent < SKI_IN_MAX_ALT
                {
                    if let Some(piste_names) = node_piste_names.get(&node_id) {
                        for piste_name in piste_names {
                            let entry = closest_by_piste
                                .entry(piste_name.clone())
                                .or_insert((d, node_id));
                            if d < entry.0 {
                                *entry = (d, node_id);
                            }
                        }
                    }
                }
            }

            // One ski-in edge per reachable piste, from the closest exit node.
            for (_, (_, source_id)) in closest_by_piste {
                let source = nodes[source_id].coord;
                let id = segments.len();
                segments.push(Segment {
                    id,
                    from: source_id,
                    to: base_id,
                    name: "ski-in".to_string(),
                    kind: "ski-in".to_string(),
                    difficulty: "-".to_string(),
                    coords: vec![source, base],
                    occupancy: None,
                    duration_min: None,
                });
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

    // Invariant required by Dijkstra: node IDs must equal their Vec index so
    // that dist[node_id] and prev[node_id] are valid direct-index accesses.
    // This holds by construction (IDs are assigned as `nodes.len()` before
    // push), but an explicit check catches any future regression immediately.
    debug_assert!(
        nodes.iter().enumerate().all(|(i, n)| n.id == i),
        "node IDs are not contiguous: Dijkstra dist[] indexing would be wrong"
    );

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

/// Return the arrival zone for `goal_node` (a lift base).
///
/// Contains `goal_node` itself plus every node that has a direct `ski-in` edge
/// into `goal_node`.  Dijkstra stops as soon as any zone node is settled.
///
/// Traverse edges are intentionally excluded: because they carry a 10x distance
/// penalty, Dijkstra always prefers continuing down the piste to the ski-in
/// source node rather than branching off via a costly traverse.  Including
/// traverse sources would let Dijkstra stop on an upper piste node that happens
/// to be within traverse range, leaving the bottom of the piste unhighlighted.
pub fn arrival_zone(goal_node: usize, segments: &[Segment]) -> HashSet<usize> {
    let mut zone = HashSet::new();
    zone.insert(goal_node);
    for seg in segments {
        if seg.to == goal_node && seg.kind == "ski-in" {
            zone.insert(seg.from);
        }
    }
    zone
}
