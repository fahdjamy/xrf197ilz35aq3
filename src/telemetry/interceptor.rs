use crate::{generate_request_id, RequestId, CLIENT_REQ_ID};
use tracing::field::Field;
use tracing::span::Attributes;
use tracing::Id;
use tracing_subscriber::layer::Context;

pub struct RequestIdVisitor {
    pub request_id: Option<String>,
}

impl RequestIdVisitor {
    pub fn new() -> Self {
        RequestIdVisitor { request_id: None }
    }
}

impl tracing::field::Visit for RequestIdVisitor {
    fn record_bool(&mut self, field: &Field, _: bool) {
        if field.name() == CLIENT_REQ_ID {
            // boolean fields are not allowed as request ids so we mark found as false
            self.request_id = None;
        }
    }

    // We expect the requestId to be a string
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "requestId" || field.name() == CLIENT_REQ_ID {
            self.request_id = Some(value.to_string());
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == CLIENT_REQ_ID {
            self.request_id = Some(format!("{:?}", value));
        }
    }
}

// Custom layer for adding request-ids to the logs
#[derive(Debug, Clone)]
pub struct RequestIdInterceptorLayer;

impl<S> tracing_subscriber::Layer<S> for RequestIdInterceptorLayer
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    // Notifies this layer that a new span was constructed with the given Attributes and ID.
    // on_new_span is called exactly once when the span is initially constructed
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("Span not found, this is a bug");
        let mut extensions = span.extensions_mut();

        extensions.insert(RequestIdInterceptorLayer);

        let mut visitor = RequestIdVisitor { request_id: None };

        attrs.record(&mut visitor);
        // If we find a request_id in the span attributes, assign it
        let request_id = if let Some(request_id) = visitor.request_id {
            request_id
        } else {
            // Generate a new ID if not provided
            // TODO: Only generate for spans that look like top-level requests.
            // Check the span name or a specific field.
            // Get a more robust check based on the span naming conventions.
            if span.name().contains("top-span-request") {
                // Example check
                generate_request_id()
            } else {
                // Not a request span or no ID provided, do nothing further
                generate_request_id()
            }
        };

        extensions.insert(RequestId(request_id));
    }
}
