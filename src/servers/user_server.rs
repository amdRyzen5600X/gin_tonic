use std::pin::Pin;

use tokio_stream::{Stream, wrappers::ReceiverStream};
use tonic::Status;
use tracing::{error, info};

use crate::{
    grpc::{
        CreateUserRequest, CreateUserResponse, DeleteUserRequest, DeleteUserResponse,
        GetUserByIdRequest, GetUserByIdResponse, GetUserByNameRequest, GetUserByNameResponse,
        GetUsersRequest, GetUsersResponse, StreamUsersRequest, StreamUsersResponse,
        UpdateUserRequest, UpdateUserResponse, user_service_server::UserService,
    },
    usecases::UserUsecaseTrait,
};

pub struct UserServer<T: UserUsecaseTrait> {
    span: tracing::Span,
    usecase: T,
}

impl<T: UserUsecaseTrait> UserServer<T> {
    pub fn new(span: tracing::Span, usecase: T) -> Self {
        Self { span, usecase }
    }
}

#[tonic::async_trait]
impl<T: UserUsecaseTrait + 'static> UserService for UserServer<T> {
    type StreamUsersStream =
        Pin<Box<dyn Stream<Item = Result<StreamUsersResponse, Status>> + Send>>;

    async fn create_user(
        &self,
        input: tonic::Request<CreateUserRequest>,
    ) -> Result<tonic::Response<CreateUserResponse>, Status> {
        let _guard = self.span.enter();
        let (_meta_data, _extentions, body) = input.into_parts();
        info!(
            "creating user with name={:?} and surname={:?}",
            body.name, body.surname
        );
        let res = self
            .usecase
            .create_user(body.name, body.surname)
            .await
            .map_err(|e| {
                let msg = format!("failed to create user: {:?}", e);
                error!(msg);
                Status::internal(msg)
            })?;
        Ok(tonic::Response::new(res))
    }

    async fn get_user_by_id(
        &self,
        input: tonic::Request<GetUserByIdRequest>,
    ) -> Result<tonic::Response<GetUserByIdResponse>, tonic::Status> {
        let _guard = self.span.enter();
        let (_meta_data, _extentions, body) = input.into_parts();
        info!("getting user by id={:?}", body.id);
        let res = self.usecase.get_user_by_id(body.id).await.map_err(|e| {
            let msg = format!("failed to retrieve user: {:?}", e);
            error!(msg);
            match e {
                crate::Error::NotFound => Status::not_found(msg),
                _ => Status::internal(msg),
            }
        })?;
        Ok(tonic::Response::new(res))
    }

    async fn get_user_by_name(
        &self,
        input: tonic::Request<GetUserByNameRequest>,
    ) -> Result<tonic::Response<GetUserByNameResponse>, tonic::Status> {
        let _guard = self.span.enter();
        let (_meta_data, _extentions, body) = input.into_parts();
        info!("getting user by name={:?}", body.name);
        let res = self
            .usecase
            .get_user_by_name(body.name)
            .await
            .map_err(|e| {
                let msg = format!("failed to retrieve user: {:?}", e);
                error!(msg);
                match e {
                    crate::Error::NotFound => Status::not_found(msg),
                    _ => Status::internal(msg),
                }
            })?;
        Ok(tonic::Response::new(res))
    }

    async fn update_user(
        &self,
        input: tonic::Request<UpdateUserRequest>,
    ) -> Result<tonic::Response<UpdateUserResponse>, tonic::Status> {
        let _guard = self.span.enter();
        let (_meta_data, _extentions, body) = input.into_parts();
        info!(
            "updating user with id={:?}, setting name={:?} and surname={:?}",
            body.id, body.name, body.surname
        );
        let res = self
            .usecase
            .update_user(body.id, body.name, body.surname)
            .await
            .map_err(|e| {
                let msg = format!("failed to update user: {:?}", e);
                error!(msg);
                Status::internal(msg)
            })?;
        Ok(tonic::Response::new(res))
    }

    async fn get_users(
        &self,
        _input: tonic::Request<GetUsersRequest>,
    ) -> Result<tonic::Response<GetUsersResponse>, tonic::Status> {
        let _guard = self.span.enter();
        info!("getting all users");
        let res = self.usecase.get_users().await.map_err(|e| {
            let msg = format!("failed to retrieve users: {:?}", e);
            error!(msg);
            Status::internal(msg)
        })?;
        Ok(tonic::Response::new(res))
    }

    async fn delete_user(
        &self,
        input: tonic::Request<DeleteUserRequest>,
    ) -> Result<tonic::Response<DeleteUserResponse>, tonic::Status> {
        let _guard = self.span.enter();
        let (_meta_data, _extentions, body) = input.into_parts();
        info!("deleting user with id={:?}", body.id);
        let res = self.usecase.delete_user(body.id).await.map_err(|e| {
            let msg = format!("failed to delete user: {:?}", e);
            error!(msg);
            Status::internal(msg)
        })?;
        Ok(tonic::Response::new(res))
    }

    async fn stream_users(
        &self,
        _input: tonic::Request<StreamUsersRequest>,
    ) -> Result<tonic::Response<Self::StreamUsersStream>, Status> {
        let _guard = self.span.enter();
        info!("streaming all users");
        let (tx, rx) = tokio::sync::mpsc::channel(128);
        self.usecase.send_users(tx).await.map_err(|e| {
            let msg = format!("failed to start streaming users: {:?}", e);
            error!(msg);
            Status::internal(msg)
        })?;

        Ok(tonic::Response::new(
            Box::pin(ReceiverStream::new(rx)) as Self::StreamUsersStream
        ))
    }
}
