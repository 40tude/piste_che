// Rust guideline compliant 2026-02-16
use std::collections::{HashMap, HashSet};

use anyhow::{Result, bail};

use crate::types::{Element, OverpassResponse};

/// Patch elevation values into nodes of an `OverpassResponse` in-place.
///
/// Each `(node_id, elevation)` pair from `elevations` is matched to the
/// corresponding `Node` element by ID.
pub fn merge_elevation(response: &mut OverpassResponse, elevations: Vec<(u64, f32)>) {
    let ele_map: HashMap<u64, f32> = elevations.into_iter().collect();

    for element in &mut response.elements {
        if let Element::Node(node) = element
            && let Some(&ele) = ele_map.get(&node.id)
        {
            node.ele = Some(ele);
        }
    }
}

/// Validate that every node referenced by a Way has an elevation value.
///
/// Collects all node IDs referenced by Way elements and checks that each
/// corresponding Node has `ele = Some(_)`.
///
/// # Errors
///
/// Returns an error listing the missing node IDs if any way-referenced node
/// has `ele = None` or is absent from the elements list.
pub fn validate_completeness(response: &OverpassResponse) -> Result<()> {
    // Collect all node IDs referenced by ways.
    let mut required: HashSet<u64> = HashSet::new();
    for element in &response.elements {
        if let Element::Way(way) = element {
            required.extend(way.nodes.iter().copied());
        }
    }

    // Index nodes by ID.
    let node_ele: HashMap<u64, Option<f32>> = response
        .elements
        .iter()
        .filter_map(|e| {
            if let Element::Node(n) = e {
                Some((n.id, n.ele))
            } else {
                None
            }
        })
        .collect();

    // Identify missing or unelevated nodes.
    let mut missing: Vec<u64> = required
        .iter()
        .filter(|id| node_ele.get(*id).copied().flatten().is_none())
        .copied()
        .collect();

    if !missing.is_empty() {
        missing.sort_unstable();
        bail!(
            "Post-merge validation failed: {} nodes missing elevation [id: {}]",
            missing.len(),
            missing.iter().map(std::string::ToString::to_string).collect::<Vec<_>>().join(", ")
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Node, Osm3s, Way};

    fn make_response(way_nodes: Vec<u64>, nodes: Vec<(u64, Option<f32>)>) -> OverpassResponse {
        let mut elements: Vec<Element> = vec![Element::Way(Way {
            id: 1,
            nodes: way_nodes,
            tags: HashMap::new(),
        })];
        for (id, ele) in nodes {
            elements.push(Element::Node(Node { id, lat: 0.0, lon: 0.0, ele }));
        }
        OverpassResponse {
            version: 0.6,
            generator: "test".to_owned(),
            osm3s: Osm3s {
                timestamp_osm_base: "".to_owned(),
                copyright: "".to_owned(),
            },
            elements,
        }
    }

    #[test]
    fn validate_completeness_returns_err_for_missing_elevation() {
        let response = make_response(vec![10, 11, 12], vec![
            (10, Some(1000.0)),
            (11, None),       // missing
            (12, Some(1200.0)),
        ]);
        let err = validate_completeness(&response).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("11"), "expected missing node ID 11 in: {msg}");
    }

    #[test]
    fn validate_completeness_passes_when_all_nodes_have_elevation() {
        let response = make_response(vec![10, 11], vec![
            (10, Some(1000.0)),
            (11, Some(1100.0)),
        ]);
        assert!(validate_completeness(&response).is_ok());
    }
}
