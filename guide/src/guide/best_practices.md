{{#include links.md}}

# Best Practices

To make it easy to test an application, it is recommended that you expose a function that configures the default set of services. This will make it simple to use the same default configuration as the application and replace only the parts that are necessary for testing.

If a service can be replaced, then it should be registered using [`try_add`]. A service can still be replaced after it has been registered, but [`try_add`] will skip the process altogether if the service has already been registered.

```rust
use crate::*;
use axum::{
    async_trait,
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::net::TcpListener;
use uuid::Uuid;

// provide a function that can be called with the expected set of services
fn add_default_services(services: &mut ServiceCollection) {
    services.try_add(ExampleUserRepo::scoped());
}

// provide a function that can build a router representing the application
fn build_app(services: ServiceCollection) -> Router {
    Router::new()
        .route("/users/:id", get(one_user))
        .route("/users", post(new_user))
        .with_provider(services.build_provider().unwrap())
}

#[tokio::main]
async fn main() {
    let mut services = ServiceCollection::new();

    add_default_services(&mut services);

    let app = build_app(services);
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    
    println!("listening on: {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

#[async_trait]
trait UserRepo {
    async fn find(&self, user_id: Uuid) -> Result<User, UserRepoError>;
    async fn create(&self, params: CreateUser) -> Result<User, UserRepoError>;
}

#[injectable(UserRepo + Send + Sync)]
struct ExampleUserRepo;

#[async_trait]
impl UserRepo for ExampleUserRepo {
    async fn find(&self, _user_id: Uuid) -> Result<User, UserRepoError> {
        unimplemented!()
    }

    async fn create(&self, _params: CreateUser) -> Result<User, UserRepoError> {
        unimplemented!()
    }
}

async fn one_user(
    Path(id): Path<Uuid>,
    Inject(repo): Inject<dyn UserRepo + Send + Sync>,
) -> Result<Json<User>, AppError> {
    let user = repo.find(user_id).await?;
    Ok(user.into())
}

async fn new_user(
    Inject(repo): Inject<dyn UserRepo + Send + Sync>,
    Json(params): Json<CreateUser>,
) -> Result<Json<User>, AppError> {
    let user = repo.create(params).await?;
    Ok(user.into())
}
```

You can now easily test your application by replacing on the only necessary services. In the following test, we:

1. Create a `TestUserRepo` to simulate the behavior of a `dyn UserRepo`
2. Register `TestUserRepo` in a new `ServiceCollection`
3. Register all other default services
   - Since `dyn UserRepo` has been registered as `TestUserRepo` and [`try_add`] was used, the default registration is skipped
4. Create a `Router` representing the application
5. Run the application with a test client
6. Invoke the HTTP `GET` method to return a single `User`

```rust
use super::*;
use crate::*;
use di::*;

#[tokio::test]
async fn get_should_return_user() {
    // arrange
    #[injectable(UserRepo + Send + Sync)]
    struct TestUserRepo;

    #[async_trait]
    impl UserRepo for TestUserRepo {
        async fn find(&self, _user_id: Uuid) -> Result<User, UserRepoError> {
            Ok(User::default())
        }

        async fn create(&self, _params: CreateUser) -> Result<User, UserRepoError> {
            unimplemented!()
        }
    }

    let mut services = ServiceCollection::new();

    services.add(TestUserRepo::scoped());
    add_default_services(services);

    let app = build_app(services);
    let client = TestClient::new(app);

    // act
    let response = client.get("/user/b51565c273c04bb4ac179232c90b20af").send().await;

    // assert
    assert_eq!(response.status(), StatusCode::OK);
}
```