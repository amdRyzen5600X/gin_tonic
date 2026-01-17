use sqlx::PgPool;

use crate::repositories::user_repository_trait::UserRepository as UserRepositoryTrait;
use crate::{Error, entities::users::User};
use async_trait::async_trait;

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    async fn create_user(&self, name: String, surname: String) -> Result<User, crate::Error> {
        let res = sqlx::query!(
            r#"
                INSERT INTO users (name, surname)
                VALUES ($1, $2)
                RETURNING id, name, surname
            "#,
            name,
            surname
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Internal(Box::new(e)))?;

        Ok(User {
            id: res.id,
            name: res.name,
            surname: res.surname,
        })
    }

    async fn get_users(&self) -> Result<(Vec<User>, i32), crate::Error> {
        let res = sqlx::query!(
            r#"
                SELECT id, name, surname
                FROM users
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Internal(Box::new(e)))?
        .into_iter()
        .map(|row| User {
            id: row.id,
            name: row.name,
            surname: row.surname,
        })
        .collect::<Vec<User>>();
        let count = res.len();

        Ok((res, count as i32))
    }

    async fn get_users_batch(&self, offset: i32, limit: i32) -> Result<Vec<User>, crate::Error> {
        let res = sqlx::query!(
            r#"
                SELECT id, name, surname
                FROM users
                ORDER BY id
                LIMIT $1 OFFSET $2
            "#,
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Internal(Box::new(e)))?
        .into_iter()
        .map(|row| User {
            id: row.id,
            name: row.name,
            surname: row.surname,
        })
        .collect();

        Ok(res)
    }

    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>, crate::Error> {
        let res = sqlx::query!(
            r#"
                SELECT id, name, surname
                FROM users
                WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await;

        match res {
            Ok(res) => Ok(Some(User {
                id: res.id,
                name: res.name,
                surname: res.surname,
            })),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(Error::Internal(Box::new(e))),
        }
    }

    async fn get_user_by_name(&self, name: String) -> Result<Option<User>, crate::Error> {
        let res = sqlx::query!(
            r#"
                SELECT id, name, surname
                FROM users
                WHERE name = $1
            "#,
            name
        )
        .fetch_one(&self.pool)
        .await;

        match res {
            Ok(res) => Ok(Some(User {
                id: res.id,
                name: res.name,
                surname: res.surname,
            })),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(Error::Internal(Box::new(e))),
        }
    }

    async fn update_user(
        &self,
        id: i32,
        name: Option<String>,
        surname: Option<String>,
    ) -> Result<Option<User>, crate::Error> {
        let res = sqlx::query!(
            r#"
                UPDATE users
                SET
                    name = COALESCE($1, name),
                    surname = COALESCE($2, surname)
                WHERE id = $3
                RETURNING id, name, surname
            "#,
            name,
            surname,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Internal(Box::new(e)))?;

        Ok(res.map(|r| User {
            id: r.id,
            name: r.name,
            surname: r.surname,
        }))
    }

    async fn delete_user(&self, id: i32) -> Result<(), crate::Error> {
        let result = sqlx::query!(
            r#"
                DELETE FROM users
                WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Internal(Box::new(e)))?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    async fn setup_pool() -> PgPool {
        dotenv::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@0.0.0.0:5432/user_service".to_string()
        });

        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database")
    }

    #[tokio::test]
    async fn test_create_user() {
        let pool = setup_pool().await;
        let repo = UserRepository::new(pool);

        let name = "Test".to_string();
        let surname = "User".to_string();

        let result = repo.create_user(name.clone(), surname.clone()).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.name, name);
        assert_eq!(user.surname, surname);
    }

    #[tokio::test]
    async fn test_get_user_by_id() {
        let pool = setup_pool().await;
        let repo = UserRepository::new(pool);

        let created = repo
            .create_user("GetById".to_string(), "Test".to_string())
            .await
            .unwrap();

        let result = repo.get_user_by_id(created.id).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert!(user.is_some());
        assert_eq!(user.unwrap().id, created.id);
    }

    #[tokio::test]
    async fn test_get_user_by_id_not_found() {
        let pool = setup_pool().await;
        let repo = UserRepository::new(pool);

        let result = repo.get_user_by_id(99999).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_user_by_name() {
        let pool = setup_pool().await;
        let repo = UserRepository::new(pool);

        let name = "ByName".to_string();
        repo.create_user(name.clone(), "Test".to_string())
            .await
            .unwrap();

        let result = repo.get_user_by_name(name.clone()).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert!(user.is_some());
        assert_eq!(user.unwrap().name, name);
    }

    #[tokio::test]
    async fn test_get_users() {
        let pool = setup_pool().await;
        let repo = UserRepository::new(pool);

        repo.create_user("User1".to_string(), "Surname1".to_string())
            .await
            .unwrap();
        repo.create_user("User2".to_string(), "Surname2".to_string())
            .await
            .unwrap();

        let result = repo.get_users().await;

        assert!(result.is_ok());
        let (users, count) = result.unwrap();
        assert!(users.len() >= 2);
        assert_eq!(count as usize, users.len());
    }

    #[tokio::test]
    async fn test_get_users_batch() {
        let pool = setup_pool().await;
        let repo = UserRepository::new(pool);

        repo.create_user("Batch1".to_string(), "User".to_string())
            .await
            .unwrap();
        repo.create_user("Batch2".to_string(), "User".to_string())
            .await
            .unwrap();

        let result = repo.get_users_batch(0, 10).await;

        assert!(result.is_ok());
        let users = result.unwrap();
        assert!(users.len() >= 2);
    }

    #[tokio::test]
    async fn test_update_user() {
        let pool = setup_pool().await;
        let repo = UserRepository::new(pool);

        let created = repo
            .create_user("Update".to_string(), "Me".to_string())
            .await
            .unwrap();

        let new_name = "Updated".to_string();
        let result = repo
            .update_user(created.id, Some(new_name.clone()), None)
            .await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert!(user.is_some());
        assert_eq!(user.unwrap().name, new_name);
    }

    #[tokio::test]
    async fn test_update_user_not_found() {
        let pool = setup_pool().await;
        let repo = UserRepository::new(pool);

        let result = repo.update_user(99999, Some("No".to_string()), None).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_delete_user() {
        let pool = setup_pool().await;
        let repo = UserRepository::new(pool);

        let created = repo
            .create_user("Delete".to_string(), "Me".to_string())
            .await
            .unwrap();

        let result = repo.delete_user(created.id).await;

        assert!(result.is_ok());

        let check = repo.get_user_by_id(created.id).await.unwrap();
        assert!(check.is_none());
    }

    #[tokio::test]
    async fn test_delete_user_not_found() {
        let pool = setup_pool().await;
        let repo = UserRepository::new(pool);

        let result = repo.delete_user(99999).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::NotFound));
    }
}
