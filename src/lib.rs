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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotFound => write!(f, "resource not found"),
            Error::Internal(e) => write!(f, "internal error: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Internal(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}
