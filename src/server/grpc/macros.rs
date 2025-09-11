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
/// *Con*: Prop drilling: Because we are passing requestId from a high-level component down to
/// a deeply nested child component/function. i.e. passing it through all the intermediate components
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

/// trace_request_with_request_id_in_local: creates a request id and sets the request id in
/// the local REQUEST_ID variable only accessible to a local thread. RAII
///
/// *Pro*: Ambient Context, data is set once at a high level, and any deeply nested function can
/// access it directly, bypassing the intermediate functions. The opposite of prop drilling.
macro_rules! trace_request_with_request_id_in_local {
    ($req:expr, $span_name:expr, $body:expr) => {
        // Mocking the request object for this example
        let request_metadata = $req.metadata();

        // Extract request ID from metadata
        let request_id = request_metadata
            .get(REQUEST_ID_KEY)
            .and_then(|id| id.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| generate_request_id());

        // Create a span with the request_id as a field for observability.
        let span = info_span!($span_name, request_id = request_id.clone());
        let _enter = span.enter();

        // Use `REQUEST_ID.scope` to make the ID available to any function
        // called within this block.
        REQUEST_ID.scope(request_id, $body).await
    };
}

pub(crate) use {trace_and_get_id, trace_request, trace_request_with_request_id_in_local};
