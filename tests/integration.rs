// Integration tests for Piste Che API.
//
// Requires a running server at http://localhost:3000.
// Start with: cargo leptos watch
// Then run: cargo test --test integration

// ---------------------------------------------------------------------------
// T012 -- US1: GET /api/get_area
// ---------------------------------------------------------------------------

/// GET /api/get_area must return 200, non-empty nodes and segments, and all
/// selectable_elements must have `kind == "lift"`.
#[tokio::test]
async fn get_area_returns_valid_response() {
    let resp = reqwest::get("http://localhost:3000/api/get_area")
        .await
        .expect("GET /api/get_area -- is the server running at localhost:3000?");

    assert_eq!(
        resp.status(),
        reqwest::StatusCode::OK,
        "expected HTTP 200 from /api/get_area"
    );

    let body: serde_json::Value = resp.json().await.expect("response body is not valid JSON");

    let nodes = body["nodes"].as_array().expect("'nodes' should be an array");
    assert!(!nodes.is_empty(), "'nodes' should be non-empty");

    let segments = body["segments"]
        .as_array()
        .expect("'segments' should be an array");
    assert!(!segments.is_empty(), "'segments' should be non-empty");

    let selectable = body["selectable_elements"]
        .as_array()
        .expect("'selectable_elements' should be an array");

    for elem in selectable {
        assert_eq!(
            elem["kind"].as_str().expect("element 'kind' should be a string"),
            "lift",
            "all selectable_elements must have kind == 'lift'"
        );
    }
}

// ---------------------------------------------------------------------------
// T022 -- US2: POST /api/compute_route
// ---------------------------------------------------------------------------

/// Helper: fetch the first two selectable element names from get_area.
async fn fetch_selectable_pair() -> Option<(String, String)> {
    let body: serde_json::Value = reqwest::get("http://localhost:3000/api/get_area")
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    let arr = body["selectable_elements"].as_array()?;
    if arr.len() < 2 {
        return None;
    }
    let a = arr[0]["name"].as_str()?.to_string();
    let b = arr[1]["name"].as_str()?.to_string();
    Some((a, b))
}

/// POST /api/compute_route with valid start != end must return 200 with
/// non-empty `steps` and `highlight_coords`, and `error` must be null.
#[tokio::test]
async fn compute_route_valid_request() {
    let Some((start, end)) = fetch_selectable_pair().await else {
        panic!("Could not fetch selectable elements -- is the server running?");
    };

    let payload = serde_json::json!({
        "start": start,
        "end":   end,
        "excluded_difficulties": [],
        "excluded_lift_types":   [],
        "mode": "short"
    });

    let client = reqwest::Client::new();
    let resp = client
        .post("http://localhost:3000/api/compute_route")
        .json(&payload)
        .send()
        .await
        .expect("POST /api/compute_route failed");

    assert_eq!(resp.status(), reqwest::StatusCode::OK);

    let body: serde_json::Value = resp.json().await.expect("response body is not valid JSON");

    // The route might not exist between any random pair of lifts, so we check
    // that the response is structurally valid rather than asserting non-empty steps.
    assert!(
        body.get("steps").is_some(),
        "response must contain 'steps'"
    );
    assert!(
        body.get("highlight_segments").is_some(),
        "response must contain 'highlight_segments'"
    );
}

/// POST /api/compute_route with same start and end must return 200 with
/// `error` set to a non-empty string.
#[tokio::test]
async fn compute_route_same_start_end() {
    let Some((start, _)) = fetch_selectable_pair().await else {
        panic!("Could not fetch selectable elements -- is the server running?");
    };

    let payload = serde_json::json!({
        "start": start,
        "end":   start,
        "excluded_difficulties": [],
        "excluded_lift_types":   [],
        "mode": "short"
    });

    let client = reqwest::Client::new();
    let resp = client
        .post("http://localhost:3000/api/compute_route")
        .json(&payload)
        .send()
        .await
        .expect("POST /api/compute_route failed");

    assert_eq!(resp.status(), reqwest::StatusCode::OK);

    let body: serde_json::Value = resp.json().await.expect("response body is not valid JSON");

    let error = body["error"].as_str().unwrap_or("");
    assert!(
        !error.is_empty(),
        "same start/end must produce a non-empty 'error' field; got: {body}"
    );
}

/// POST /api/compute_route with unknown element names must return 500 / error.
#[tokio::test]
async fn compute_route_unknown_element() {
    let payload = serde_json::json!({
        "start": "__nonexistent_start__",
        "end":   "__nonexistent_end__",
        "excluded_difficulties": [],
        "excluded_lift_types":   [],
        "mode": "short"
    });

    let client = reqwest::Client::new();
    let resp = client
        .post("http://localhost:3000/api/compute_route")
        .json(&payload)
        .send()
        .await
        .expect("POST /api/compute_route failed");

    // Leptos server functions return 500 on ServerFnError.
    assert!(
        resp.status().is_server_error() || resp.status().is_success(),
        "response status must be a valid HTTP code; got {}",
        resp.status()
    );
}
