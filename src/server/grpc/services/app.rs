use crate::context::ApplicationContext;
use crate::grpc_services::app_service_server::AppService;
use crate::grpc_services::{CheckHealthRequest, CheckHealthResponse};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct AppServiceManager {
    app_ctx: Arc<ApplicationContext>,
}

impl AppServiceManager {
    pub fn new(app_ctx: Arc<ApplicationContext>) -> Self {
        AppServiceManager { app_ctx }
    }
}

#[tonic::async_trait]
impl AppService for AppServiceManager {
    async fn check_health(
        &self,
        _: Request<CheckHealthRequest>,
    ) -> Result<Response<CheckHealthResponse>, Status> {
        Ok(Response::new(CheckHealthResponse {
            is_up: true,
            app_id: self.app_ctx.app_id.clone().to_string(),
            region: self.app_ctx.block_region.clone().to_string(),
        }))
    }
}
