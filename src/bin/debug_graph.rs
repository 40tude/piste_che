// CLI binary for visual graph debugging.
// Reuses the routing pipeline to generate a self-contained Leaflet.js HTML map.
//
// Edit the constants below, then: cargo run --bin debug_graph
// Output: temp/debug_graph.html

use anyhow::{Context, Result};
use piste_che::routing::{
    Node, OsmData, Segment, adjacency_from_segments, arrival_zone, build_graph, dijkstra,
    data::find_latest_json,
};
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::path::Path;

// ---------------------------------------------------------------------------
// Config -- edit these and re-run
// ---------------------------------------------------------------------------

/// Start/end node IDs for optional Dijkstra route overlay.
/// Set both to `Some(id)` to display a route; `None` skips routing.
const START_NODE: Option<usize> = None;
const END_NODE: Option<usize> = None;

/// Toggle visibility of synthetic edge layers.
const SHOW_TRAVERSES: bool = true;
const SHOW_SKI_IN: bool = true;
const SHOW_SKI_OUT: bool = true;

/// Draw circles at lift base/exit nodes with this radius (metres).
/// `None` hides the radius layer entirely.
const RADIUS_DISPLAY: Option<f64> = None;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> Result<()> {
    let path = find_latest_json(Path::new("data")).context("Selecting latest data file")?;
    println!("Data file: {}", path.display());

    let osm = OsmData::load(&path).with_context(|| format!("Loading {}", path.display()))?;
    let (nodes, segments, _route_elements) = build_graph(&osm);
    let adj = adjacency_from_segments(&segments);

    println!("Nodes: {}  Segments: {}", nodes.len(), segments.len());

    let route_segment_ids: Vec<usize> = match (START_NODE, END_NODE) {
        (Some(start), Some(end)) => {
            let goal_zone = arrival_zone(end);
            let route =
                dijkstra(start, &goal_zone, nodes.len(), &segments, &adj, &[], &[])
                    .unwrap_or_default();
            println!("Route: {} segments", route.len());
            route
        }
        _ => Vec::new(),
    };

    let html = generate_html(&nodes, &segments, &route_segment_ids);

    std::fs::create_dir_all("temp").context("Creating temp directory")?;
    std::fs::write("temp/debug_graph.html", html).context("Writing HTML")?;
    println!("Written: temp/debug_graph.html");

    Ok(())
}

// ---------------------------------------------------------------------------
// GeoJSON helpers
// ---------------------------------------------------------------------------

/// Convert `[lat, lon, ele]` to GeoJSON `[lon, lat]`.
fn coord_to_geojson(c: &[f64; 3]) -> String {
    format!("[{}, {}]", c[1], c[0])
}

/// One segment as a GeoJSON Feature with LineString geometry.
fn segment_to_geojson_feature(seg: &Segment) -> String {
    let coords: Vec<String> = seg.coords.iter().map(coord_to_geojson).collect();
    let name_json = serde_json::to_string(&seg.name).unwrap_or_default();
    let diff_json = serde_json::to_string(&seg.difficulty).unwrap_or_default();
    let kind_json = serde_json::to_string(&seg.kind).unwrap_or_default();
    format!(
        concat!(
            r#"{{"type":"Feature","properties":{{"id":{},"name":{},"kind":{},"difficulty":{},"from":{},"to":{}}},"#,
            r#""geometry":{{"type":"LineString","coordinates":[{}]}}}}"#,
        ),
        seg.id,
        name_json,
        kind_json,
        diff_json,
        seg.from,
        seg.to,
        coords.join(","),
    )
}

/// One node as a GeoJSON Feature with Point geometry.
fn node_to_geojson_feature(node: &Node, connections: &str) -> String {
    let conn_json = serde_json::to_string(connections).unwrap_or_default();
    format!(
        concat!(
            r#"{{"type":"Feature","properties":{{"id":{},"lat":{:.6},"lon":{:.6},"ele":{:.0},"connections":{}}},"#,
            r#""geometry":{{"type":"Point","coordinates":[{}, {}]}}}}"#,
        ),
        node.id,
        node.coord[0],
        node.coord[1],
        node.coord[2],
        conn_json,
        node.coord[1],
        node.coord[0],
    )
}

fn difficulty_color(difficulty: &str) -> &str {
    match difficulty {
        "novice" | "easy" => "#22c55e",
        "intermediate" => "#3b82f6",
        "advanced" => "#ef4444",
        "expert" => "#111827",
        "freeride" => "#a855f7",
        _ => "#9ca3af",
    }
}

// ---------------------------------------------------------------------------
// HTML generation
// ---------------------------------------------------------------------------

#[expect(clippy::too_many_lines, reason = "HTML template requires sequential string building")]
fn generate_html(nodes: &[Node], segments: &[Segment], route_segment_ids: &[usize]) -> String {
    // -- Partition segment features by kind --
    let mut piste_features = String::new();
    let mut lift_features = String::new();
    let mut traverse_features = String::new();
    let mut ski_in_features = String::new();
    let mut ski_out_features = String::new();

    for seg in segments {
        let feature = segment_to_geojson_feature(seg);
        let target = match seg.kind.as_str() {
            "piste" => &mut piste_features,
            "lift" => &mut lift_features,
            "traverse" => &mut traverse_features,
            "ski-in" => &mut ski_in_features,
            "ski-out" => &mut ski_out_features,
            _ => continue,
        };
        if !target.is_empty() {
            target.push(',');
        }
        target.push_str(&feature);
    }

    // -- Node connections lookup --
    let mut node_connections: HashMap<usize, Vec<String>> = HashMap::new();
    for seg in segments {
        node_connections
            .entry(seg.from)
            .or_default()
            .push(format!("-> {} ({})", seg.name, seg.kind));
        node_connections
            .entry(seg.to)
            .or_default()
            .push(format!("<- {} ({})", seg.name, seg.kind));
    }

    // -- Node features --
    let mut node_features = String::new();
    for node in nodes {
        let conns = node_connections
            .get(&node.id)
            .map(|v| v.join(", "))
            .unwrap_or_default();
        if !node_features.is_empty() {
            node_features.push(',');
        }
        node_features.push_str(&node_to_geojson_feature(node, &conns));
    }

    // -- Route features --
    let route_set: HashSet<usize> = route_segment_ids.iter().copied().collect();
    let mut route_features = String::new();
    for seg in segments {
        if route_set.contains(&seg.id) {
            if !route_features.is_empty() {
                route_features.push(',');
            }
            route_features.push_str(&segment_to_geojson_feature(seg));
        }
    }

    // -- Radius circles JS --
    let radius_js = if let Some(r) = RADIUS_DISPLAY {
        let mut lift_node_ids: HashSet<usize> = HashSet::new();
        for seg in segments {
            if seg.kind == "lift" {
                lift_node_ids.insert(seg.from);
                lift_node_ids.insert(seg.to);
            }
        }
        let mut js = String::from("var radiusLayer = L.layerGroup().addTo(map);\n");
        for node in nodes {
            if lift_node_ids.contains(&node.id) {
                let _ = writeln!(
                    js,
                    "L.circle([{}, {}], {{radius:{r},color:'#f97316',weight:1,fillOpacity:0.08}}).addTo(radiusLayer);",
                    node.coord[0], node.coord[1],
                );
            }
        }
        js
    } else {
        String::new()
    };

    let radius_checkbox = if RADIUS_DISPLAY.is_some() {
        r#"<label><input type="checkbox" checked data-layer="radius"> Radius</label>"#
    } else {
        ""
    };

    let radius_layer_entry = if RADIUS_DISPLAY.is_some() {
        ", radius: radiusLayer"
    } else {
        ""
    };

    let traverse_checked = if SHOW_TRAVERSES { " checked" } else { "" };
    let ski_in_checked = if SHOW_SKI_IN { " checked" } else { "" };
    let ski_out_checked = if SHOW_SKI_OUT { " checked" } else { "" };

    let traverse_add = if SHOW_TRAVERSES {
        ".addTo(map)"
    } else {
        ""
    };
    let ski_in_add = if SHOW_SKI_IN { ".addTo(map)" } else { "" };
    let ski_out_add = if SHOW_SKI_OUT { ".addTo(map)" } else { "" };

    // -- Difficulty color map for JS --
    let mut diff_colors_js = String::from("{");
    for (diff, color) in [
        ("novice", difficulty_color("novice")),
        ("easy", difficulty_color("easy")),
        ("intermediate", difficulty_color("intermediate")),
        ("advanced", difficulty_color("advanced")),
        ("expert", difficulty_color("expert")),
        ("freeride", difficulty_color("freeride")),
    ] {
        let _ = write!(diff_colors_js, "'{diff}':'{color}',");
    }
    diff_colors_js.push('}');

    // -- Assemble HTML --
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Debug Graph</title>
<link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css"/>
<script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"></script>
<style>
html,body,#map{{height:100%;margin:0}}
#controls{{position:absolute;top:10px;right:10px;z-index:1000;background:#fff;padding:10px 14px;border-radius:6px;box-shadow:0 2px 8px rgba(0,0,0,.25);font:13px/1.6 system-ui,sans-serif}}
#controls label{{display:block;cursor:pointer;white-space:nowrap}}
</style>
</head>
<body>
<div id="map"></div>
<div id="controls">
<b>Layers</b>
<label><input type="checkbox" checked data-layer="pistes"> Pistes</label>
<label><input type="checkbox" checked data-layer="lifts"> Lifts</label>
<label><input type="checkbox"{traverse_checked} data-layer="traverses"> Traverses</label>
<label><input type="checkbox"{ski_in_checked} data-layer="ski_in"> Ski-in</label>
<label><input type="checkbox"{ski_out_checked} data-layer="ski_out"> Ski-out</label>
<label><input type="checkbox" checked data-layer="nodes"> Nodes</label>
<label><input type="checkbox" checked data-layer="route"> Route</label>
{radius_checkbox}
</div>
<script>
var map=L.map('map');
L.tileLayer('https://{{s}}.tile.openstreetmap.org/{{z}}/{{x}}/{{y}}.png',{{attribution:'OSM'}}).addTo(map);

var diffColors={diff_colors_js};

var pisteLayer=L.geoJSON({{"type":"FeatureCollection","features":[{piste_features}]}},{{
  style:function(f){{return{{color:diffColors[f.properties.difficulty]||'#9ca3af',weight:3,opacity:0.8}}}},
  onEachFeature:function(f,l){{l.bindPopup('Seg '+f.properties.id+': '+f.properties.name+'<br>'+f.properties.kind+' / '+f.properties.difficulty+'<br>from '+f.properties.from+' to '+f.properties.to)}}
}}).addTo(map);

var liftLayer=L.geoJSON({{"type":"FeatureCollection","features":[{lift_features}]}},{{
  style:function(){{return{{color:'#f59e0b',weight:3,opacity:0.9}}}},
  onEachFeature:function(f,l){{l.bindPopup('Seg '+f.properties.id+': '+f.properties.name+'<br>'+f.properties.kind+' / '+f.properties.difficulty+'<br>from '+f.properties.from+' to '+f.properties.to)}}
}}).addTo(map);

var traverseLayer=L.geoJSON({{"type":"FeatureCollection","features":[{traverse_features}]}},{{
  style:function(){{return{{color:'#6b7280',weight:2,opacity:0.7,dashArray:'6 4'}}}},
  onEachFeature:function(f,l){{l.bindPopup('Seg '+f.properties.id+': traverse<br>from '+f.properties.from+' to '+f.properties.to)}}
}}){traverse_add};

var skiInLayer=L.geoJSON({{"type":"FeatureCollection","features":[{ski_in_features}]}},{{
  style:function(){{return{{color:'#06b6d4',weight:2,opacity:0.8}}}},
  onEachFeature:function(f,l){{l.bindPopup('Seg '+f.properties.id+': ski-in<br>from '+f.properties.from+' to '+f.properties.to)}}
}}){ski_in_add};

var skiOutLayer=L.geoJSON({{"type":"FeatureCollection","features":[{ski_out_features}]}},{{
  style:function(){{return{{color:'#8b5cf6',weight:2,opacity:0.8}}}},
  onEachFeature:function(f,l){{l.bindPopup('Seg '+f.properties.id+': ski-out<br>from '+f.properties.from+' to '+f.properties.to)}}
}}){ski_out_add};

var nodeLayer=L.geoJSON({{"type":"FeatureCollection","features":[{node_features}]}},{{
  pointToLayer:function(f,ll){{return L.circleMarker(ll,{{radius:4,color:'#1e3a5f',weight:1,fillColor:'#3b82f6',fillOpacity:0.7}})}},
  onEachFeature:function(f,l){{l.bindPopup('Node '+f.properties.id+'<br>lat='+f.properties.lat+' lon='+f.properties.lon+' ele='+f.properties.ele+'m<br>'+f.properties.connections)}}
}}).addTo(map);

var routeLayer=L.geoJSON({{"type":"FeatureCollection","features":[{route_features}]}},{{
  style:function(){{return{{color:'#ec4899',weight:5,opacity:0.9}}}},
  onEachFeature:function(f,l){{l.bindPopup('Route seg '+f.properties.id+': '+f.properties.name)}}
}}).addTo(map);

{radius_js}

var bounds=nodeLayer.getBounds();
if(bounds.isValid())map.fitBounds(bounds,{{padding:[20,20]}});

var layers={{pistes:pisteLayer,lifts:liftLayer,traverses:traverseLayer,ski_in:skiInLayer,ski_out:skiOutLayer,nodes:nodeLayer,route:routeLayer{radius_layer_entry}}};
document.querySelectorAll('#controls input').forEach(function(cb){{
  cb.addEventListener('change',function(){{
    var layer=layers[this.dataset.layer];
    if(layer){{this.checked?map.addLayer(layer):map.removeLayer(layer)}}
  }});
}});
</script>
</body>
</html>"##
    )
}
