use std::env;

use gin_tonik::{
    grpc::user_service_server::UserServiceServer, repositories::user_repository::UserRepository,
    servers::user_server::UserServer, usecases::user_usecase::UserUsecase,
};
use tonic::transport::Server;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:42069".parse().unwrap();

    let span = tracing::span!(Level::INFO, "UserService");

    let db_url = env::var("DATABASE_URL")
        .unwrap_or("postgres://postgres:postgres@0.0.0.0:5432/user_service".to_owned());
    let Ok(connection) = sqlx::postgres::PgPool::connect(&db_url).await else {
        panic!("AAAAA cannot connect ot db");
    };

    let user_repo = UserRepository::new(connection);
    let user_usecase = UserUsecase::new(user_repo);
    let user_server = UserServer::new(span, user_usecase);

    tracing_subscriber::fmt().pretty().init();

    Server::builder()
        .add_service(UserServiceServer::new(user_server))
        .serve(addr)
        .await?;

    Ok(())
}
