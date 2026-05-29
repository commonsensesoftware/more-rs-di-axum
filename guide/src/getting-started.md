# Getting Started

The simplest way to get started is to install the crate using the default features.

```bash
cargo add more-di-axum
```

## Example

The following example reworks the `axum` [dependency injection example] with full dependency injection support using
[more-di].


```rust
use async_trait::async_trait;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use di::ServiceCollection;
use di_axum::{prelude::*, Inject};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::net::TcpListener;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let provider = ServiceCollection::new()
        .add(ExampleUserRepo::singleton())
        .build_provider()
        .unwrap();

    let app = Router::new()
        .route("/users/:id", get(one_user))
        .route("/users", post(new_user))
        .with_provider(provider);

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    
    println!("listening on: {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn one_user(
    Path(id): Path<Uuid>,
    Inject(repo): Inject<dyn UserRepo>,
) -> Result<Json<User>, AppError> {
    let user = repo.find(user_id).await?;
    Ok(user.into())
}

async fn new_user(
    Inject(repo): Inject<dyn UserRepo>,
    Json(params): Json<CreateUser>,
) -> Result<Json<User>, AppError> {
    let user = repo.create(params).await?;
    Ok(user.into())
}

#[derive(Debug)]
enum UserRepoError {
    NotFound,
    InvalidUserName,
}

enum AppError {
    UserRepo(UserRepoError),
}

#[async_trait]
trait UserRepo: Send + Sync {
    async fn find(&self, user_id: Uuid) -> Result<User, UserRepoError>;
    async fn create(&self, params: CreateUser) -> Result<User, UserRepoError>;
}

#[derive(Debug, Serialize)]
struct User {
    id: Uuid,
    username: String,
}

#[derive(Debug, Deserialize)]
struct CreateUser {
    username: String,
}

#[injectable(UserRepo)]
struct ExampleUserRepo;

#[async_trait]
impl UserRepo for ExampleUserRepo {
    async fn find(&self, user_id: Uuid) -> Result<User, UserRepoError> {
        unimplemented!()
    }

    async fn create(&self, params: CreateUser) -> Result<User, UserRepoError> {
        unimplemented!()
    }
}

impl From<UserRepoError> for AppError {
    fn from(inner: UserRepoError) -> Self {
        AppError::UserRepo(inner)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::UserRepo(UserRepoError::NotFound) => {
                (StatusCode::NOT_FOUND, "User not found")
            }
            AppError::UserRepo(UserRepoError::InvalidUserName) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "Invalid user name")
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
```

[dependency injection example]: https://github.com/tokio-rs/axum/tree/main/examples/dependency-injection/src
[more-di]: https://crates.io/crates/more-di