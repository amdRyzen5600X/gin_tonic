use tokio::sync::mpsc::Sender;
use tonic::Status;
use tracing::error;
use tracing::info;

use crate::{
    grpc::{
        CreateUserResponse, DeleteUserResponse, GetUserByIdResponse, GetUserByNameResponse,
        GetUsersResponse, StreamUsersResponse, UpdateUserResponse,
    },
    repositories::UserRepository,
    usecases::UserUsecaseTrait,
};
use async_trait::async_trait;

pub struct UserUsecase<T: UserRepository + Clone> {
    repo: T,
}

impl<T: UserRepository + Clone> UserUsecase<T> {
    pub fn new(repo: T) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl<T: UserRepository + Clone + 'static> UserUsecaseTrait for UserUsecase<T> {
    async fn create_user(
        &self,
        name: String,
        surname: String,
    ) -> Result<CreateUserResponse, crate::Error> {
        let res = self.repo.create_user(name, surname).await?;
        Ok(CreateUserResponse {
            user: Some(crate::grpc::User {
                id: res.id,
                name: res.name,
                surname: res.surname,
            }),
        })
    }

    async fn get_users(&self) -> Result<GetUsersResponse, crate::Error> {
        let (res, count) = self.repo.get_users().await?;

        Ok(GetUsersResponse {
            users: res
                .iter()
                .map(|u| crate::grpc::User {
                    id: u.id,
                    name: u.name.clone(),
                    surname: u.surname.clone(),
                })
                .collect(),
            count,
        })
    }

    async fn get_user_by_id(&self, id: i32) -> Result<GetUserByIdResponse, crate::Error> {
        let res = self.repo.get_user_by_id(id).await?;

        if let Some(user) = res {
            Ok(GetUserByIdResponse {
                user: Some(crate::grpc::User {
                    id: user.id,
                    name: user.name.clone(),
                    surname: user.surname.clone(),
                }),
            })
        } else {
            Err(crate::Error::NotFound)
        }
    }

    async fn get_user_by_name(&self, name: String) -> Result<GetUserByNameResponse, crate::Error> {
        let res = self.repo.get_user_by_name(name).await?;

        if let Some(user) = res {
            Ok(GetUserByNameResponse {
                user: Some(crate::grpc::User {
                    id: user.id,
                    name: user.name.clone(),
                    surname: user.surname.clone(),
                }),
            })
        } else {
            Err(crate::Error::NotFound)
        }
    }

    async fn update_user(
        &self,
        id: i32,
        name: Option<String>,
        surname: Option<String>,
    ) -> Result<UpdateUserResponse, crate::Error> {
        let res = self.repo.update_user(id, name, surname).await?;

        if let Some(u) = res {
            Ok(UpdateUserResponse {
                user: Some(crate::grpc::User {
                    id: u.id,
                    name: u.name,
                    surname: u.surname,
                }),
            })
        } else {
            Err(crate::Error::NotFound)
        }
    }

    async fn delete_user(&self, id: i32) -> Result<DeleteUserResponse, crate::Error> {
        self.repo.delete_user(id).await?;

        Ok(DeleteUserResponse {})
    }

    async fn send_users(
        &self,
        tx: Sender<Result<StreamUsersResponse, Status>>,
    ) -> Result<(), crate::Error> {
        const BATCH_SIZE: i32 = 100;
        let repo = self.repo.clone();

        tokio::spawn(async move {
            let span = tracing::info_span!("streaming users");
            let _guard = span.enter();

            let mut offset = 0;

            loop {
                let batch = repo.get_users_batch(offset, BATCH_SIZE).await;

                match batch {
                    Ok(users) if users.is_empty() => break,
                    Ok(users) => {
                        for user in users {
                            let res = StreamUsersResponse {
                                user: Some(crate::grpc::User {
                                    id: user.id,
                                    name: user.name,
                                    surname: user.surname,
                                }),
                            };

                            if (tx.send(Ok(res))).await.is_err() {
                                info!("client disconnected");
                                break;
                            }
                        }
                        offset += BATCH_SIZE;
                    }
                    Err(e) => {
                        error!("error fetching users batch: {:?}", e);
                        break;
                    }
                }
            }

            info!("streaming complete");
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::users::User;
    use mockall::predicate::*;

    mockall::mock! {
        Repo {}

        #[async_trait::async_trait]
        impl crate::repositories::user_repository_trait::UserRepository for Repo {
            async fn create_user(&self, name: String, surname: String) -> Result<User, crate::Error>;
            async fn get_users(&self) -> Result<(Vec<User>, i32), crate::Error>;
            async fn get_users_batch(&self, offset: i32, limit: i32) -> Result<Vec<User>, crate::Error>;
            async fn get_user_by_id(&self, id: i32) -> Result<Option<User>, crate::Error>;
            async fn get_user_by_name(&self, name: String) -> Result<Option<User>, crate::Error>;
            async fn update_user(&self, id: i32, name: Option<String>, surname: Option<String>) -> Result<Option<User>, crate::Error>;
            async fn delete_user(&self, id: i32) -> Result<(), crate::Error>;
        }
    }

    impl Clone for MockRepo {
        fn clone(&self) -> Self {
            Self::default()
        }
    }

    #[tokio::test]
    async fn test_create_user() {
        let mut mock_repo = MockRepo::new();
        mock_repo
            .expect_create_user()
            .with(eq("John".to_string()), eq("Doe".to_string()))
            .times(1)
            .returning(|name, surname| {
                Ok(User {
                    id: 1,
                    name,
                    surname,
                })
            });

        let usecase = UserUsecase::new(mock_repo);
        let result = usecase
            .create_user("John".to_string(), "Doe".to_string())
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.user.is_some());
        assert_eq!(response.user.unwrap().id, 1);
    }

    #[tokio::test]
    async fn test_get_users() {
        let mut mock_repo = MockRepo::new();
        mock_repo.expect_get_users().times(1).returning(|| {
            Ok((
                vec![
                    User {
                        id: 1,
                        name: "John".to_string(),
                        surname: "Doe".to_string(),
                    },
                    User {
                        id: 2,
                        name: "Jane".to_string(),
                        surname: "Smith".to_string(),
                    },
                ],
                2,
            ))
        });

        let usecase = UserUsecase::new(mock_repo);
        let result = usecase.get_users().await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.users.len(), 2);
        assert_eq!(response.count, 2);
    }

    #[tokio::test]
    async fn test_get_user_by_id_found() {
        let mut mock_repo = MockRepo::new();
        mock_repo
            .expect_get_user_by_id()
            .with(eq(1))
            .times(1)
            .returning(|_| {
                Ok(Some(User {
                    id: 1,
                    name: "John".to_string(),
                    surname: "Doe".to_string(),
                }))
            });

        let usecase = UserUsecase::new(mock_repo);
        let result = usecase.get_user_by_id(1).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.user.is_some());
        assert_eq!(response.user.unwrap().id, 1);
    }

    #[tokio::test]
    async fn test_get_user_by_id_not_found() {
        let mut mock_repo = MockRepo::new();
        mock_repo
            .expect_get_user_by_id()
            .with(eq(999))
            .times(1)
            .returning(|_| Ok(None));

        let usecase = UserUsecase::new(mock_repo);
        let result = usecase.get_user_by_id(999).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::Error::NotFound));
    }

    #[tokio::test]
    async fn test_get_user_by_name_found() {
        let mut mock_repo = MockRepo::new();
        mock_repo
            .expect_get_user_by_name()
            .with(eq("John".to_string()))
            .times(1)
            .returning(|_| {
                Ok(Some(User {
                    id: 1,
                    name: "John".to_string(),
                    surname: "Doe".to_string(),
                }))
            });

        let usecase = UserUsecase::new(mock_repo);
        let result = usecase.get_user_by_name("John".to_string()).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.user.is_some());
        assert_eq!(response.user.unwrap().name, "John");
    }

    #[tokio::test]
    async fn test_get_user_by_name_not_found() {
        let mut mock_repo = MockRepo::new();
        mock_repo
            .expect_get_user_by_name()
            .with(eq("Unknown".to_string()))
            .times(1)
            .returning(|_| Ok(None));

        let usecase = UserUsecase::new(mock_repo);
        let result = usecase.get_user_by_name("Unknown".to_string()).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::Error::NotFound));
    }

    #[tokio::test]
    async fn test_update_user_found() {
        let mut mock_repo = MockRepo::new();
        mock_repo
            .expect_update_user()
            .with(eq(1), eq(Some("Updated".to_string())), eq(None))
            .times(1)
            .returning(|_, name, _| {
                Ok(Some(User {
                    id: 1,
                    name: name.unwrap(),
                    surname: "Doe".to_string(),
                }))
            });

        let usecase = UserUsecase::new(mock_repo);
        let result = usecase
            .update_user(1, Some("Updated".to_string()), None)
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.user.is_some());
        assert_eq!(response.user.unwrap().name, "Updated");
    }

    #[tokio::test]
    async fn test_update_user_not_found() {
        let mut mock_repo = MockRepo::new();
        mock_repo
            .expect_update_user()
            .with(eq(999), eq(Some("No".to_string())), eq(None))
            .times(1)
            .returning(|_, _, _| Ok(None));

        let usecase = UserUsecase::new(mock_repo);
        let result = usecase.update_user(999, Some("No".to_string()), None).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::Error::NotFound));
    }

    #[tokio::test]
    async fn test_delete_user() {
        let mut mock_repo = MockRepo::new();
        mock_repo
            .expect_delete_user()
            .with(eq(1))
            .times(1)
            .returning(|_| Ok(()));

        let usecase = UserUsecase::new(mock_repo);
        let result = usecase.delete_user(1).await;

        assert!(result.is_ok());
    }
}
