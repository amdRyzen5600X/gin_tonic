use sqlx::PgPool;

use crate::{Error, entities::users::User};

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn create_user(&self, name: String, surname: String) -> Result<User, crate::Error> {
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

    pub async fn get_users(&self) -> Result<(Vec<User>, i32), crate::Error> {
        let res = sqlx::query!(
            r#"
                SELECT id, name, surname
                FROM users
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Internal(Box::new(e)))?
        .iter()
        .map(|row| User {
            id: row.id,
            name: row.name.clone(),
            surname: row.surname.clone(),
        })
        .collect::<Vec<User>>();
        let count = res.len();

        Ok((res, count as i32))
    }

    pub async fn get_user_by_id(&self, id: i32) -> Result<Option<User>, crate::Error> {
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

    pub async fn get_user_by_name(&self, name: String) -> Result<Option<User>, crate::Error> {
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

    pub async fn update_user(
        &self,
        id: i32,
        name: Option<String>,
        surname: Option<String>,
    ) -> Result<Option<User>, crate::Error> {
        let user = self.get_user_by_id(id).await?;
        if let Some(user) = user {
            let name = name.unwrap_or(user.name);
            let surname = surname.unwrap_or(user.surname);
            let res = sqlx::query!(
                r#"
                        UPDATE users
                        SET name = $1, surname = $2
                        WHERE id = $3
                        RETURNING id, name, surname
                    "#,
                name,
                surname,
                id
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Internal(Box::new(e)))?;
            return Ok(Some(User {
                id: res.id,
                name: res.name,
                surname: res.surname,
            }));
        }
        Ok(None)
    }

    pub async fn delete_user(&self, id: i32) -> Result<(), crate::Error> {
        sqlx::query!(
            r#"
                DELETE FROM users
                WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Internal(Box::new(e)))?;
        Ok(())
    }
}
