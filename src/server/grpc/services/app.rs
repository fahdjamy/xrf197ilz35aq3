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
        request: Request<CheckHealthRequest>,
    ) -> Result<Response<CheckHealthResponse>, Status> {
        todo!()
    }
}
