use crate::context::{ApplicationContext, UserContext};
use crate::error::OrchestrateError;
use crate::grpc_services::account_service_server::AccountService;
use crate::grpc_services::{CreateAccountRequest, CreateAccountResponse};
use crate::server::grpc::header::get_xrf_user_auth_header;
use crate::server::grpc::interceptors::trace_request;
use crate::{create_account, generate_request_id, REQUEST_ID_KEY, XRF_USER_FINGERPRINT};
use cassandra_cpp::Session;
use sqlx::PgPool;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{info, info_span};

pub struct AccountServiceManager {
    pg_pool: Arc<PgPool>,
    cassandra_session: Arc<Session>,
    app_ctx: Arc<ApplicationContext>,
}

impl AccountServiceManager {
    pub fn new(
        pg_pool: Arc<PgPool>,
        cassandra_session: Arc<Session>,
        app_ctx: Arc<ApplicationContext>,
    ) -> Self {
        AccountServiceManager {
            pg_pool,
            app_ctx,
            cassandra_session,
        }
    }
}

#[tonic::async_trait]
impl AccountService for AccountServiceManager {
    async fn create_account(
        &self,
        request: Request<CreateAccountRequest>,
    ) -> Result<Response<CreateAccountResponse>, Status> {
        trace_request!(request, "create_account");
        let user_fp = get_xrf_user_auth_header(&request.metadata(), XRF_USER_FINGERPRINT)?;
        let req = request.into_inner();

        info!(
            "creating new account :: (name={}, user_fp={})",
            &req.acct_type, &user_fp
        );
        let user_ctx = UserContext::load_user_context(user_fp, req.timezone.clone(), None, None);

        let _ = create_account(
            &self.pg_pool,
            req.currency,
            req.acct_type,
            &user_ctx,
            &self.cassandra_session,
            &self.app_ctx,
        )
        .await
        .map_err(|err| match err {
            OrchestrateError::NotFoundError(err) => {
                Status::not_found(format!("not found: {}", err))
            }
            OrchestrateError::InvalidArgument(err) => {
                Status::invalid_argument(format!("Invalid argument: {}", err))
            }
            _ => Status::internal("Internal server error"),
        })?;

        unimplemented!()
    }
}
