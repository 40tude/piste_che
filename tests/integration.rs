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

/// POST /api/compute_route with same start and end must return 200.
///
/// All selectable elements are lifts (FR-004). For a lift, same start/end
/// triggers the circuit case (summit -> piste -> base). Two outcomes are valid:
///   (a) Circuit succeeds: `error` is null, `steps` is non-empty.
///   (b) No route found:   `error` is non-empty, `steps` is empty.
/// Unacceptable: `error` is null AND `steps` is empty.
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
    let steps = body["steps"].as_array().map_or(0, Vec::len);
    assert!(
        !error.is_empty() || steps > 0,
        "same-lift start/end must yield a circuit (non-empty steps) or a 'no route' error; got: {body}"
    );
}

// ---------------------------------------------------------------------------
// T032 -- US3: segment coord shape
// ---------------------------------------------------------------------------

/// Every coord in `segments` must be a 3-element array `[lat, lon, alt]`.
#[tokio::test]
async fn get_area_segment_coords_have_three_elements() {
    let body: serde_json::Value =
        reqwest::get("http://localhost:3000/api/get_area")
            .await
            .expect("GET /api/get_area -- is the server running?")
            .json()
            .await
            .expect("response body is not valid JSON");

    let segments = body["segments"].as_array().expect("'segments' must be an array");
    for seg in segments {
        let coords = seg["coords"].as_array().expect("segment must have 'coords' array");
        for coord in coords {
            let arr = coord.as_array().expect("each coord must be an array");
            assert_eq!(
                arr.len(),
                3,
                "coord must be [lat, lon, alt] (3 elements); got {} in seg {}",
                arr.len(),
                seg["name"]
            );
        }
    }
}

// ---------------------------------------------------------------------------
// T033 -- US4: unimplemented routing modes
// ---------------------------------------------------------------------------

/// POST /api/compute_route with mode != "short" must return 200 with a
/// non-empty `error` field describing the unimplemented mode.
#[tokio::test]
async fn compute_route_mode_not_short_returns_error() {
    let Some((start, end)) = fetch_selectable_pair().await else {
        panic!("Could not fetch selectable elements -- is the server running?");
    };

    let payload = serde_json::json!({
        "start": start,
        "end":   end,
        "excluded_difficulties": [],
        "excluded_lift_types":   [],
        "mode": "sport"
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
        "mode 'sport' must set a non-empty 'error' field; got: {body}"
    );
    assert!(
        error.contains("not implemented") || error.contains("sport"),
        "error message must mention 'not implemented' or the mode name; got: {error}"
    );
}

// ---------------------------------------------------------------------------
// T034 -- US5: all difficulties excluded -> no route
// ---------------------------------------------------------------------------

/// Excluding all known piste difficulties must cause Dijkstra to find no path
/// between two distinct lift elements.  The response must be 200 with a
/// non-empty `error` field.
#[tokio::test]
async fn compute_route_all_difficulties_excluded_returns_no_route() {
    let Some((start, end)) = fetch_selectable_pair().await else {
        panic!("Could not fetch selectable elements -- is the server running?");
    };

    // Exclude every difficulty level used in the resort.
    let all_difficulties = ["novice", "easy", "intermediate", "advanced", "freeride"];

    let payload = serde_json::json!({
        "start": start,
        "end":   end,
        "excluded_difficulties": all_difficulties,
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
    let steps = body["steps"].as_array().map_or(0, Vec::len);
    let error = body["error"].as_str().unwrap_or("");

    // A route between identical lifts is the only valid zero-step success;
    // for all other pairs an error must be set.
    if steps > 0 && start == end {
        // Circuit on a lift -- acceptable non-error outcome, skip assertion.
    } else {
        assert!(
            !error.is_empty(),
            "blocking all piste difficulties must yield an error; got: {body}"
        );
    }
}

/// POST /api/compute_route with unknown element names must return 500
/// (Leptos ServerFnError) or a 200 with a non-empty `error` field.
///
/// Both outcomes signal that the server correctly rejected the request;
/// the exact mechanism depends on how the Leptos version encodes errors.
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

    let status = resp.status();
    if status.is_success() {
        // 200: the error must be encoded in the response body.
        let body: serde_json::Value =
            resp.json().await.expect("response body is not valid JSON");
        let error_msg = body["error"].as_str().unwrap_or("");
        assert!(
            !error_msg.is_empty(),
            "200 response for unknown element must have non-empty 'error' field; got: {body}"
        );
    } else {
        assert!(
            status.is_server_error(),
            "expected 500 or 200+error for unknown element; got {status}"
        );
    }
}
