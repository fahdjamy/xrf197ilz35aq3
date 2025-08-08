use crate::context::{ApplicationContext, UserContext};
use crate::error::OrchestrateError;
use crate::grpc_services::account_service_server::AccountService;
use crate::grpc_services::{
    AccountResponse, CreateAccountRequest, CreateAccountResponse, GetUserAccountRequest,
    GetUserAccountResponse, GetUserAccountsRequest, GetUserAccountsResponse,
    GetWalletHoldingRequest, GetWalletHoldingResponse, WalletResponse,
};
use crate::server::grpc::header::get_xrf_user_auth_header;
use crate::server::grpc::interceptors::trace_request;
use crate::{
    create_account, generate_request_id, get_user_accounts_by_currencies_and_types,
    DEFAULT_TIMEZONE, REQUEST_ID_KEY, XRF_USER_FINGERPRINT,
};
use cassandra_cpp::Session;
use prost_types::Timestamp;
use rust_decimal::prelude::ToPrimitive;
use sqlx::PgPool;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{error, info, info_span, warn};

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
        let event = "createUserAccount";
        trace_request!(request, "create_account");
        let user_fp = get_xrf_user_auth_header(&request.metadata(), XRF_USER_FINGERPRINT)?;
        let req = request.into_inner();

        info!(
            "creating new account :: (name={}, user_fp={})",
            &req.acct_type, &user_fp
        );
        let user_ctx = UserContext::load_user_context(user_fp, req.timezone.clone(), None, None);

        let (created_acct, created_wallet) = create_account(
            &self.pg_pool,
            req.currency,
            req.acct_type,
            &user_ctx,
            &self.cassandra_session,
            &self.app_ctx,
        )
        .await
        .map_err(|err| map_orchestrator_err_to_grpc_error(event, err))?;

        Ok(Response::new(CreateAccountResponse {
            account: Some(AccountResponse {
                locked: created_acct.locked,
                status: created_acct.status.to_string(),
                account_id: created_acct.id.to_string(),
                account_type: created_acct.account_type.to_string(),
                creation_time: Some(Timestamp {
                    seconds: created_acct.creation_time.timestamp(),
                    nanos: created_acct.creation_time.timestamp_subsec_nanos() as i32,
                }),
                modification_time: Some(Timestamp {
                    seconds: created_acct.modification_time.timestamp(),
                    nanos: created_acct.modification_time.timestamp_subsec_nanos() as i32,
                }),
                wallet_holding: Some(WalletResponse {
                    balance: f32::try_from(created_wallet.balance).unwrap_or_else(|er| {
                        warn!("Err converting balance : err={}, defaulted to 0.0", er);
                        0.0
                    }),
                    currency: created_wallet.currency.to_string(),
                    modification_time: Some(Timestamp {
                        seconds: created_wallet.modification_time.timestamp(),
                        nanos: created_wallet.modification_time.timestamp_subsec_nanos() as i32,
                    }),
                }),
            }),
        }))
    }

    async fn get_user_account(
        &self,
        request: Request<GetUserAccountRequest>,
    ) -> Result<Response<GetUserAccountResponse>, Status> {
        unimplemented!()
    }

    async fn get_user_accounts(
        &self,
        request: Request<GetUserAccountsRequest>,
    ) -> Result<Response<GetUserAccountsResponse>, Status> {
        let event = "get_user_accounts";
        trace_request!(request, "get_user_account");
        let user_fp = get_xrf_user_auth_header(&request.metadata(), XRF_USER_FINGERPRINT)?;
        let req = request.into_inner();
        let user_ctx =
            UserContext::load_user_context(user_fp, DEFAULT_TIMEZONE.to_string(), None, None);

        let currencies = match req.currencies {
            None => vec![],
            Some(currency_list) => currency_list.currencies.iter().cloned().collect(),
        };
        let acct_types = match req.acct_types {
            None => vec![],
            Some(acct_types_list) => acct_types_list.types.iter().cloned().collect(),
        };

        let saved_accounts_and_wallet = get_user_accounts_by_currencies_and_types(
            &self.pg_pool,
            &currencies,
            &acct_types,
            &user_ctx,
        )
        .await
        .map_err(|err| map_orchestrator_err_to_grpc_error(event, err))?;

        let account_resp: Vec<AccountResponse> = saved_accounts_and_wallet
            .iter()
            .map(|(acct, wallet)| AccountResponse {
                locked: acct.locked,
                status: acct.status.to_string(),
                account_id: acct.id.to_string(),
                account_type: acct.account_type.to_string(),
                creation_time: Some(Timestamp {
                    seconds: acct.creation_time.timestamp(),
                    nanos: acct.creation_time.timestamp_subsec_nanos() as i32,
                }),
                modification_time: Some(Timestamp {
                    seconds: acct.modification_time.timestamp(),
                    nanos: acct.modification_time.timestamp_subsec_nanos() as i32,
                }),
                wallet_holding: Some(WalletResponse {
                    currency: wallet.currency.to_string(),
                    balance: wallet.balance.to_f32().unwrap_or_default(),
                    modification_time: Some(Timestamp {
                        seconds: wallet.modification_time.timestamp(),
                        nanos: wallet.modification_time.timestamp_subsec_nanos() as i32,
                    }),
                }),
            })
            .collect();

        Ok(Response::new(GetUserAccountsResponse {
            accounts: account_resp,
        }))
    }

    async fn get_wallet_holding(
        &self,
        request: Request<GetWalletHoldingRequest>,
    ) -> Result<Response<GetWalletHoldingResponse>, Status> {
        unimplemented!()
    }
}

fn map_orchestrator_err_to_grpc_error(event: &str, err: OrchestrateError) -> Status {
    match err {
        OrchestrateError::InvalidArgument(_) => {
            Status::invalid_argument(format!("Invalid argument: {}", err))
        }
        OrchestrateError::NotFoundError(err) => Status::not_found(format!("Not found: {}", err)),
        OrchestrateError::DatabaseError(err) => {
            error!("event={} :: database error: {}", event, err);
            Status::internal("Internal server error")
        }
        OrchestrateError::RowConstraintViolation(err) => {
            error!("event={} :: row constraint err :: err={}", event, err);
            Status::internal("Internal server error")
        }
        _ => Status::internal("Internal server error"),
    }
}
