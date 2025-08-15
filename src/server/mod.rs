mod grpc;
mod server;

pub mod grpc_services {
    tonic::include_proto!("proto.xrfq3.v1");
    tonic::include_proto!("proto.account.v1");
}
pub use server::GrpcServer;
