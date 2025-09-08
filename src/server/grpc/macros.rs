/// trace_request creates a request id but does not return it
macro_rules! trace_request {
    ($req:expr, $span_name:expr) => {
        // Extract request ID from metadata
        let request_id = $req
            .metadata()
            .get(REQUEST_ID_KEY)
            .and_then(|id| id.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| generate_request_id());

        // Create span for this method, linked to the gRPC span by the request ID
        let span = info_span!($span_name, request_id = request_id);
        let _enter = span.enter();
    };
}

/// trace_and_get_id returns a request id
///
/// *Pro*: The flow of data is extremely clear. There is no "magic."
/// Any function that needs the request ID must declare it in its signature
///
/// *Con*:
macro_rules! trace_and_get_id {
    ($req:expr, $span_name:expr) => {{
        // Use a block to return the request_id
        let request_id = $req
            .metadata()
            .get(REQUEST_ID_KEY)
            .and_then(|id| id.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| generate_request_id());

        let span = info_span!($span_name, request_id = request_id.clone());
        let _enter = span.enter();

        request_id // Return the ID
    }};
}

pub(crate) use {trace_and_get_id, trace_request};
