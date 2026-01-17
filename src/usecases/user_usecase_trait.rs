use crate::{
    Error,
    grpc::{
        CreateUserResponse, DeleteUserResponse, GetUserByIdResponse, GetUserByNameResponse,
        GetUsersResponse, StreamUsersResponse, UpdateUserResponse,
    },
};
use async_trait::async_trait;
use tokio::sync::mpsc::Sender;
use tonic::Status;

#[async_trait]
pub trait UserUsecase: Send + Sync {
    async fn create_user(&self, name: String, surname: String)
    -> Result<CreateUserResponse, Error>;
    async fn get_users(&self) -> Result<GetUsersResponse, Error>;
    async fn get_user_by_id(&self, id: i32) -> Result<GetUserByIdResponse, Error>;
    async fn get_user_by_name(&self, name: String) -> Result<GetUserByNameResponse, Error>;
    async fn update_user(
        &self,
        id: i32,
        name: Option<String>,
        surname: Option<String>,
    ) -> Result<UpdateUserResponse, Error>;
    async fn delete_user(&self, id: i32) -> Result<DeleteUserResponse, Error>;
    async fn send_users(
        &self,
        tx: Sender<Result<StreamUsersResponse, Status>>,
    ) -> Result<(), Error>;
}
