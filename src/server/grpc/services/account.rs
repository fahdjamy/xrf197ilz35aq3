use crate::grpc_services::account_service_server::AccountService;
use crate::grpc_services::{CreateAccountRequest, CreateAccountResponse};
use crate::server::grpc::interceptors::trace_request;
use crate::REQUEST_ID_KEY;
use sqlx::PgPool;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::info_span;

pub struct AccountServiceManager {
    pg_pool: Arc<PgPool>,
}

impl AccountServiceManager {
    pub fn new(pg_pool: Arc<PgPool>) -> Self {
        AccountServiceManager { pg_pool }
    }
}

#[tonic::async_trait]
impl AccountService for AccountServiceManager {
    async fn create_account(
        &self,
        request: Request<CreateAccountRequest>,
    ) -> Result<Response<CreateAccountResponse>, Status> {
        trace_request!(request, "create_account");
        let _ = request.into_inner();
        unimplemented!()
    }
}
