use crate::grpc_services::account_service_server::AccountService;
use crate::grpc_services::{CreateAccountRequest, CreateAccountResponse};
use sqlx::PgPool;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct AccountServiceManager {
    pg_pool: Arc<PgPool>,
}

#[tonic::async_trait]
impl AccountService for AccountServiceManager {
    async fn create_account(
        &self,
        request: Request<CreateAccountRequest>,
    ) -> Result<Response<CreateAccountResponse>, Status> {
        unimplemented!()
    }
}
