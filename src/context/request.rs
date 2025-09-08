use crate::RequestId;
use std::fmt::Display;

#[derive(Clone, Debug)]
pub struct RequestContext {
    pub request_ip: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<RequestId>,
}
