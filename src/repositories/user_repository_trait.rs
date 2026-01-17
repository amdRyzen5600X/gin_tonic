use crate::{Error, entities::users::User};
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync + Clone {
    async fn create_user(&self, name: String, surname: String) -> Result<User, Error>;
    async fn get_users(&self) -> Result<(Vec<User>, i32), Error>;
    async fn get_users_batch(&self, offset: i32, limit: i32) -> Result<Vec<User>, Error>;
    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>, Error>;
    async fn get_user_by_name(&self, name: String) -> Result<Option<User>, Error>;
    async fn update_user(
        &self,
        id: i32,
        name: Option<String>,
        surname: Option<String>,
    ) -> Result<Option<User>, Error>;
    async fn delete_user(&self, id: i32) -> Result<(), Error>;
}
