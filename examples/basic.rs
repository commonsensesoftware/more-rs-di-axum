//! Run with
//!
//! ```not_rust
//! cargo test -p example-basic-di
//! ```

use axum::{
    routing::{get, post},
    Json, Router,
};
use di::*;
use di_axum::*;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let mut services = ServiceCollection::new();
    add_default_services(&mut services);
    let app = build_app(services);
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

trait User {
    fn greet(&self) -> &str;
}

trait Sorter {
    fn sort(&self, array: &mut [u8]);
}

#[injectable(User + Send + Sync)]
struct DefaultUser;

impl User for DefaultUser {
    fn greet(&self) -> &str {
        "Hello, World!"
    }
}

#[injectable(Sorter + Send + Sync)]
struct AscendingOrder;

impl Sorter for AscendingOrder {
    fn sort(&self, array: &mut [u8]) {
        array.sort()
    }
}

fn add_default_services(services: &mut ServiceCollection) {
    services.try_add(DefaultUser::scoped());
    services.try_add(AscendingOrder::scoped());
}

async fn greeting(Inject(user): Inject<dyn User + Send + Sync>) -> String {
    user.greet().into()
}

async fn sort_content(
    Inject(sorter): Inject<dyn Sorter + Send + Sync>,
    payload: Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let mut body: Vec<_> = payload
        .0
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u8)
        .collect();
    sorter.sort(&mut body);
    Json(serde_json::json!({ "data": body }))
}

fn build_app(services: ServiceCollection) -> Router {
    Router::new()
        .route("/", get(greeting))
        .route("/json", post(sort_content))
        .with_provider(services.build_provider().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{self, Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use serde_json::{json, Value};
    use tower::ServiceExt;

    #[injectable(User + Send + Sync)]
    struct TestUser;

    // test user with alternate greeting
    impl User for TestUser {
        fn greet(&self) -> &str {
            "I am a teapot!"
        }
    }

    #[injectable(Sorter + Send + Sync)]
    struct DescendingOrder;

    // test orderer in descending order
    impl Sorter for DescendingOrder {
        fn sort(&self, array: &mut [u8]) {
            array.sort();
            array.reverse()
        }
    }

    #[tokio::test]
    async fn get_should_return_expected_greeting() {
        // arrange
        let mut services = ServiceCollection::new();

        services.add(TestUser::scoped());
        add_default_services(&mut services);

        let app = build_app(services);

        // act
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        // assert
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"I am a teapot!");
    }

    #[tokio::test]
    async fn post_should_order_array() {
        // arrange
        let mut services = ServiceCollection::new();

        services.add(DescendingOrder::scoped());
        add_default_services(&mut services);

        let app = build_app(services);

        // act
        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/json")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(
                        serde_json::to_vec(&json!([1, 2, 3, 4])).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // assert
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, json!({ "data": [4, 3, 2, 1] }));
    }
}
