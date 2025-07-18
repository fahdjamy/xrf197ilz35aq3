mod grpc;
mod server;

pub mod grpc_services {
    tonic::include_proto!("proto.account.v1");
}
