pub mod grpc {
    tonic::include_proto!("user.v1");
}

pub mod entities;
pub mod repositories;
pub mod servers;
pub mod usecases;

#[derive(Debug)]
pub enum Error {
    NotFound,
    Internal(Box<dyn std::error::Error + Send + Sync>),
}
