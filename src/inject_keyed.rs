use axum::http::StatusCode;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use di::{KeyedRef, KeyedRefMut, ServiceProvider};
use std::any::type_name;
use std::convert::Infallible;

/// Represents a container for an optional, injected, keyed service.
#[derive(Clone, Debug)]
pub struct TryInjectWithKey<TKey, TSvc: ?Sized + 'static>(pub Option<KeyedRef<TKey, TSvc>>);

/// Represents a container for a required, injected, keyed service.
#[derive(Clone, Debug)]
pub struct InjectWithKey<TKey, TSvc: ?Sized + 'static>(pub KeyedRef<TKey, TSvc>);

/// Represents a container for an optional, mutable, injected, keyed service.
#[derive(Clone, Debug)]
pub struct TryInjectWithKeyMut<TKey, TSvc: ?Sized + 'static>(pub Option<KeyedRefMut<TKey, TSvc>>);

/// Represents a container for a required, mutable, injected, keyed service.
#[derive(Clone, Debug)]
pub struct InjectWithKeyMut<TKey, TSvc: ?Sized + 'static>(pub KeyedRefMut<TKey, TSvc>);

/// Represents a container for a collection of injected, keyed services.
#[derive(Clone, Debug)]
pub struct InjectAllWithKey<TKey, TSvc: ?Sized + 'static>(pub Vec<KeyedRef<TKey, TSvc>>);

/// Represents a container for a collection of mutable, injected, keyed services.
#[derive(Clone, Debug)]
pub struct InjectAllWithKeyMut<TKey, TSvc: ?Sized + 'static>(pub Vec<KeyedRefMut<TKey, TSvc>>);

#[inline]
fn unregistered_type_with_key<TKey, TSvc: ?Sized>() -> String {
    format!(
        "No service for type '{}' with the key '{}' has been registered.",
        type_name::<TSvc>(),
        type_name::<TKey>()
    )
}

#[async_trait]
impl<TKey, TSvc, S> FromRequestParts<S> for TryInjectWithKey<TKey, TSvc>
where
    TSvc: ?Sized + 'static,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(provider) = parts.extensions.get::<ServiceProvider>() {
            Ok(Self(provider.get_by_key::<TKey, TSvc>()))
        } else {
            Ok(Self(None))
        }
    }
}

#[async_trait]
impl<TKey, TSvc, S> FromRequestParts<S> for InjectWithKey<TKey, TSvc>
where
    TSvc: ?Sized + 'static,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(provider) = parts.extensions.get::<ServiceProvider>() {
            if let Some(service) = provider.get_by_key::<TKey, TSvc>() {
                return Ok(Self(service));
            }
        }

        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            unregistered_type_with_key::<TKey, TSvc>(),
        ))
    }
}

#[async_trait]
impl<TKey, TSvc, S> FromRequestParts<S> for TryInjectWithKeyMut<TKey, TSvc>
where
    TSvc: ?Sized + 'static,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(provider) = parts.extensions.get::<ServiceProvider>() {
            Ok(Self(provider.get_by_key_mut::<TKey, TSvc>()))
        } else {
            Ok(Self(None))
        }
    }
}

#[async_trait]
impl<TKey, TSvc, S> FromRequestParts<S> for InjectWithKeyMut<TKey, TSvc>
where
    TSvc: ?Sized + 'static,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(provider) = parts.extensions.get::<ServiceProvider>() {
            if let Some(service) = provider.get_by_key_mut::<TKey, TSvc>() {
                return Ok(Self(service));
            }
        }

        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            unregistered_type_with_key::<TKey, TSvc>(),
        ))
    }
}

#[async_trait]
impl<TKey, TSvc, S> FromRequestParts<S> for InjectAllWithKey<TKey, TSvc>
where
    TSvc: ?Sized + 'static,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(provider) = parts.extensions.get::<ServiceProvider>() {
            Ok(Self(provider.get_all_by_key::<TKey, TSvc>().collect()))
        } else {
            Ok(Self(Vec::with_capacity(0)))
        }
    }
}

#[async_trait]
impl<TKey, TSvc, S> FromRequestParts<S> for InjectAllWithKeyMut<TKey, TSvc>
where
    TSvc: ?Sized + 'static,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(provider) = parts.extensions.get::<ServiceProvider>() {
            Ok(Self(provider.get_all_by_key_mut::<TKey, TSvc>().collect()))
        } else {
            Ok(Self(Vec::with_capacity(0)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RouterServiceProviderExtensions, TestClient};
    use axum::{
        routing::{get, post},
        Router,
    };
    use di::{injectable, Injectable, ServiceCollection};
    use http::StatusCode;

    mod key {
        pub struct Basic;
        pub struct Advanced;
    }

    #[tokio::test]
    async fn request_should_fail_with_500_for_unregistered_service_with_key() {
        // arrange
        struct Service;

        impl Service {
            fn do_work(&self) -> String {
                "Test".into()
            }
        }

        async fn handler(InjectWithKey(service): InjectWithKey<key::Basic, Service>) -> String {
            service.do_work()
        }

        let app = Router::new()
            .route("/test", get(handler))
            .with_provider(ServiceProvider::default());

        let client = TestClient::new(app);

        // act
        let response = client.get("/test").send().await;

        // assert
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn try_inject_with_key_into_handler() {
        // arrange
        #[injectable]
        struct Service;

        async fn handler(
            TryInjectWithKey(_service): TryInjectWithKey<key::Advanced, Service>,
        ) -> StatusCode {
            StatusCode::NO_CONTENT
        }

        let app = Router::new()
            .route("/test", post(handler))
            .with_provider(ServiceProvider::default());

        let client = TestClient::new(app);

        // act
        let response = client.post("/test").send().await;

        // assert
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn inject_with_key_into_handler() {
        // arrange
        trait Service: Send + Sync {
            fn do_work(&self) -> String;
        }

        #[injectable(Service)]
        struct ServiceImpl;

        impl Service for ServiceImpl {
            fn do_work(&self) -> String {
                "Test".into()
            }
        }

        async fn handler(InjectWithKey(service): InjectWithKey<key::Basic, dyn Service>) -> String {
            service.do_work()
        }

        let provider = ServiceCollection::new()
            .add(ServiceImpl::scoped().with_key::<key::Basic>())
            .build_provider()
            .unwrap();

        let app = Router::new()
            .route("/test", get(handler))
            .with_provider(provider);

        let client = TestClient::new(app);

        // act
        let response = client.get("/test").send().await;
        let text = response.text().await;

        // assert
        assert_eq!(&text, "Test");
    }

    #[tokio::test]
    async fn inject_all_with_key_into_handler() {
        // arrange
        trait Thing: Send + Sync {}

        #[injectable(Thing)]
        struct Thing1;

        #[injectable(Thing)]
        struct Thing2;

        #[injectable(Thing)]
        struct Thing3;

        impl Thing for Thing1 {}
        impl Thing for Thing2 {}
        impl Thing for Thing3 {}

        async fn handler(
            InjectAllWithKey(things): InjectAllWithKey<key::Basic, dyn Thing>,
        ) -> String {
            things.len().to_string()
        }

        let provider = ServiceCollection::new()
            .try_add_to_all(Thing1::scoped().with_key::<key::Basic>())
            .try_add_to_all(Thing2::scoped().with_key::<key::Basic>())
            .try_add_to_all(Thing3::scoped().with_key::<key::Advanced>())
            .build_provider()
            .unwrap();

        let app = Router::new()
            .route("/test", get(handler))
            .with_provider(provider);

        let client = TestClient::new(app);

        // act
        let response = client.get("/test").send().await;
        let text = response.text().await;

        // assert
        assert_eq!(&text, "2");
    }
}
