use crate::context::{ApplicationContext, UserContext};
use crate::core::{Account, WalletHolding};
use crate::error::OrchestrateError;
use crate::grpc_services::account_service_server::AccountService;
use crate::grpc_services::{
    AccountResponse, CreateAccountRequest, CreateAccountResponse,
    FindAccountByCurrencyAndTypeRequest, FindAccountByCurrencyAndTypeResponse,
    FindAccountsByCurrencyOrTypeRequest, FindAccountsByCurrencyOrTypeResponse, FindWalletRequest,
    FindWalletResponse, WalletResponse,
};
use crate::server::grpc::header::get_xrf_user_auth_header;
use crate::server::grpc::interceptors::trace_request;
use crate::{
    create_account, find_account_by_currency_and_type, find_user_wallet_for_acct,
    generate_request_id, get_user_accounts_by_currencies_or_types, DEFAULT_TIMEZONE,
    REQUEST_ID_KEY, XRF_USER_FINGERPRINT,
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
    async fn find_wallet(
        &self,
        request: Request<FindWalletRequest>,
    ) -> Result<Response<FindWalletResponse>, Status> {
        let event = "getWalletHolding";
        trace_request!(request, "get_wallet_holding");
        get_xrf_user_auth_header(&request.metadata(), XRF_USER_FINGERPRINT)?;

        let req = request.into_inner();
        let wallet = find_user_wallet_for_acct(&self.pg_pool, &req.account_id)
            .await
            .map_err(|err| map_orchestrator_err_to_grpc_error(event, err))?;

        match wallet {
            None => Err(Status::not_found(format!(
                "no wallet found for account id = {}",
                &req.account_id
            ))),
            Some(found_wallet) => Ok(Response::new(FindWalletResponse {
                wallet_holding: Some(WalletResponse {
                    balance: found_wallet.balance.to_f32().unwrap_or_default(),
                    currency: found_wallet.currency.to_string(),
                    modification_time: Some(Timestamp {
                        seconds: found_wallet.modification_time.timestamp(),
                        nanos: found_wallet.modification_time.timestamp_subsec_nanos() as i32,
                    }),
                }),
            })),
        }
    }

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

        let (account, wallet) = create_account(
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
            account: Some(create_account_response(&account, &wallet)),
        }))
    }

    async fn find_accounts_by_currency_or_type(
        &self,
        request: Request<FindAccountsByCurrencyOrTypeRequest>,
    ) -> Result<Response<FindAccountsByCurrencyOrTypeResponse>, Status> {
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

        let saved_accounts_and_wallet = get_user_accounts_by_currencies_or_types(
            &self.pg_pool,
            &currencies,
            &acct_types,
            &user_ctx,
        )
        .await
        .map_err(|err| map_orchestrator_err_to_grpc_error(event, err))?;

        let account_resp: Vec<AccountResponse> = saved_accounts_and_wallet
            .iter()
            .map(|(acct, wallet)| create_account_response(&acct, &wallet))
            .collect();

        Ok(Response::new(FindAccountsByCurrencyOrTypeResponse {
            accounts: account_resp,
        }))
    }

    async fn find_account_by_currency_and_type(
        &self,
        request: Request<FindAccountByCurrencyAndTypeRequest>,
    ) -> Result<Response<FindAccountByCurrencyAndTypeResponse>, Status> {
        let event = "getUserAccount";
        trace_request!(request, "get_user_account");
        let req = request.into_inner();

        let (account, wallet) =
            match find_account_by_currency_and_type(&self.pg_pool, &req.currency, &req.acct_type)
                .await
                .map_err(|err| map_orchestrator_err_to_grpc_error(event, err))?
            {
                Some((account, wallet)) => (account, wallet),
                None => {
                    return Err(Status::not_found("no account found"));
                }
            };

        Ok(Response::new(FindAccountByCurrencyAndTypeResponse {
            account: Some(create_account_response(&account, &wallet)),
        }))
    }
}

fn map_orchestrator_err_to_grpc_error(event: &str, err: OrchestrateError) -> Status {
    match err {
        OrchestrateError::InvalidArgument(err) => Status::invalid_argument(err.to_string()),
        OrchestrateError::NotFoundError(err) => Status::not_found(format!("Not found: {}", err)),
        OrchestrateError::DatabaseError(err) => {
            error!("event={} :: database error: {}", event, err);
            Status::internal("Internal server error")
        }
        OrchestrateError::RecordAlreadyExists(err) => Status::already_exists(err.to_string()),
        _ => Status::internal("Internal server error"),
    }
}

fn create_account_response(account: &Account, wallet: &WalletHolding) -> AccountResponse {
    AccountResponse {
        locked: account.locked,
        status: account.status.to_string(),
        account_id: account.id.to_string(),
        account_type: account.account_type.to_string(),
        creation_time: Some(Timestamp {
            seconds: account.creation_time.timestamp(),
            nanos: account.creation_time.timestamp_subsec_nanos() as i32,
        }),
        modification_time: Some(Timestamp {
            seconds: account.modification_time.timestamp(),
            nanos: account.modification_time.timestamp_subsec_nanos() as i32,
        }),
        wallet_holding: Some(WalletResponse {
            balance: f32::try_from(wallet.balance).unwrap_or_else(|er| {
                warn!("Err converting balance : err={}, defaulted to 0.0", er);
                0.0
            }),
            currency: wallet.currency.to_string(),
            modification_time: Some(Timestamp {
                seconds: wallet.modification_time.timestamp(),
                nanos: wallet.modification_time.timestamp_subsec_nanos() as i32,
            }),
        }),
    }
}
