# AGENTS.md

## Project Overview

Rust 2024 edition gRPC service implementing a user management API. Uses Tonic/prost for gRPC, SQLx for PostgreSQL database access, and follows clean architecture pattern with separate layers for entities, repositories, usecases, and servers.

## Build, Lint, Test Commands

```bash
# Build
cargo build
cargo build --release

# Development check (faster than build)
cargo check

# Run tests
cargo test
cargo test --lib                    # Run library tests only
cargo test --bin gin_tonik           # Run binary tests only

# Run single test
cargo test test_name
cargo test -- --exact test_name

# Lint
cargo clippy --all-targets --all-features

# Format
cargo fmt --check                    # Check formatting without making changes
cargo fmt                            # Format all source files

# Database (requires Docker)
docker-compose up -d                 # Start PostgreSQL
docker-compose down                  # Stop PostgreSQL

# Proto compilation (automatic via build.rs)
cargo build                          # Compiles proto/service.proto automatically
```

## Architecture

```
src/
├── main.rs              # Entry point, server setup
├── lib.rs               # Library root, error types, module exports
├── entities/            # Data models
│   ├── mod.rs
│   └── users.rs
├── repositories/        # Database access layer
│   ├── mod.rs
│   └── user_repository.rs
├── usecases/            # Business logic layer
│   ├── mod.rs
│   └── user_usecase.rs
└── servers/             # gRPC server implementations
    ├── mod.rs
    └── user_server.rs

proto/service.proto     # gRPC service definition
migrations/              # SQL database migrations
```

## Code Style Guidelines

### Imports

Order imports with blank lines between groups:
1. `std::*` library imports
2. External crate imports
3. Local module imports (`crate::`)

```rust
use std::env;

use tonic::transport::Server;
use tracing::Level;

use crate::entities::users::User;
```

### Naming Conventions

- **Structs/Enums**: `PascalCase` (`UserRepository`, `Error`)
- **Functions/Methods**: `snake_case` (`create_user`, `get_user_by_id`)
- **Modules**: `snake_case` (`user_repository.rs`, `user_server.rs`)
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Type Parameters**: `T`, `E` (single uppercase letters)

### Types

Use `Option<T>` for nullable fields and `Result<T, E>` for fallible operations:

```rust
pub async fn get_user_by_id(&self, id: i32) -> Result<Option<User>, crate::Error>;
```

Derive common traits for structs:
```rust
#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode)]
pub struct User { ... }
```

### Error Handling

Project uses custom `Error` enum in `src/lib.rs`:
- `Error::NotFound` - Resource not found
- `Error::Internal(Box<dyn std::error::Error + Send + Sync>)` - Other errors

Always use `?` operator for error propagation:
```rust
let res = self.repo.create_user(name, surname).await?;
```

Map SQLx errors to custom errors:
```rust
.map_err(|e| Error::Internal(Box::new(e)))?
```

In servers, map custom errors to gRPC `Status`:
```rust
.map_err(|e| {
    match e {
        crate::Error::NotFound => Status::not_found(msg),
        _ => Status::internal(msg),
    }
})?
```

### Database

- Use `sqlx::query!` macro for compile-time checked queries
- Use `r#"..."#` raw string literals for SQL
- Pool connections with `PgPool`
- Database URL from `DATABASE_URL` environment variable

```rust
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
```

### Async Runtime

- Use `#[tokio::main]` for async main functions
- All database and gRPC operations are async
- Use `tokio::spawn` for concurrent tasks (e.g., streaming responses)

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> { ... }
```

### Logging

Use `tracing` crate for structured logging:
- Create spans for context: `tracing::span!(Level::INFO, "UserService")`
- Enter spans with `let _guard = span.enter()`
- Log levels: `info!`, `error!`, `debug!`, `warn!`

```rust
let _guard = self.span.enter();
info!("creating user with name={:?} and surname={:?}", name, surname);
error!("failed to create user: {:?}", e);
```

### gRPC/Proto

- Proto definitions in `proto/service.proto`
- Auto-compiled via `build.rs` using `tonic_prost_build`
- Service implementations use `#[tonic::async_trait]`
- Return `tonic::Response<T>` from service methods
- Streaming returns `Pin<Box<dyn Stream<Item = Result<T, Status>> + Send>>`

```rust
#[tonic::async_trait]
impl UserService for UserServer {
    type StreamUsersStream = Pin<Box<dyn Stream<Item = Result<StreamUsersResponse, Status>> + Send>>;

    async fn create_user(&self, input: tonic::Request<CreateUserRequest>)
        -> Result<tonic::Response<CreateUserResponse>, Status>
    { ... }
}
```

### Modules

- Each directory has a `mod.rs` file exporting public APIs
- Use `pub mod` to declare modules
- Use `crate::` for absolute paths within the crate

### Environment

Required environment variables (see `example.env`):
- `DATABASE_URL` - PostgreSQL connection string
- Optional: Docker Compose handles `POSTGRES_USER`, `POSTGRES_PASSWORD`, `POSTGRES_DB`

### Testing

Currently no tests exist. When adding tests:
- Use `#[cfg(test)]` for test modules
- Use `#[tokio::test]` for async test functions
- Integration tests for repositories and usecases
- Use `cargo test` to run all tests

### General Guidelines

- Prefer `let` over `let mut` when possible
- Use `.clone()` sparingly - prefer borrowing
- Pattern match with `match` or `if let` for `Result`/`Option`
- Keep functions focused and single-responsibility
- Follow Rust idioms and best practices from the Rust Book
- Code should pass `cargo clippy` without warnings
- Format with `cargo fmt` before committing
