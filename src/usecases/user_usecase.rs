use crate::{
    grpc::{
        CreateUserResponse, DeleteUserResponse, GetUserByIdResponse, GetUserByNameResponse,
        GetUsersResponse, UpdateUserResponse,
    },
    repositories::user_repository::UserRepository,
};

pub struct UserUsecase {
    repo: UserRepository,
}

impl UserUsecase {
    pub fn new(repo: UserRepository) -> Self {
        Self { repo }
    }
    pub async fn create_user(
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

    pub async fn get_users(&self) -> Result<GetUsersResponse, crate::Error> {
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

    pub async fn get_user_by_id(&self, id: i32) -> Result<GetUserByIdResponse, crate::Error> {
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

    pub async fn get_user_by_name(
        &self,
        name: String,
    ) -> Result<GetUserByNameResponse, crate::Error> {
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

    pub async fn update_user(
        &self,
        id: i32,
        name: Option<String>,
        surname: Option<String>,
    ) -> Result<UpdateUserResponse, crate::Error> {
        let res = self.repo.update_user(id, name, surname).await?;

        Ok(UpdateUserResponse {
            user: res.map(|u| crate::grpc::User {
                id: u.id,
                name: u.name.clone(),
                surname: u.surname.clone(),
            }),
        })
    }

    pub async fn delete_user(&self, id: i32) -> Result<DeleteUserResponse, crate::Error> {
        self.repo.delete_user(id).await?;

        Ok(DeleteUserResponse {})
    }
}
