use axum::http::StatusCode;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use di::{Ref, RefMut, ServiceProvider};
use std::any::type_name;
use std::convert::Infallible;

/// Represents a container for an optional, injected service.
#[derive(Clone, Debug)]
pub struct TryInject<T: ?Sized>(pub Option<Ref<T>>);

/// Represents a container for a required, injected service.
#[derive(Clone, Debug)]
pub struct Inject<T: ?Sized>(pub Ref<T>);

/// Represents a container for an optional, mutable, injected service.
#[derive(Clone, Debug)]
pub struct TryInjectMut<T: ?Sized>(pub Option<RefMut<T>>);

/// Represents a container for a required, mutable, injected service.
#[derive(Clone, Debug)]
pub struct InjectMut<T: ?Sized>(pub RefMut<T>);

/// Represents a container for a collection of injected services.
#[derive(Clone, Debug)]
pub struct InjectAll<T: ?Sized>(pub Vec<Ref<T>>);

/// Represents a container for a collection of mutable, injected services.
#[derive(Clone, Debug)]
pub struct InjectAllMut<T: ?Sized>(pub Vec<RefMut<T>>);

#[inline]
fn unregistered_type<T: ?Sized>() -> String {
    format!(
        "No service for type '{}' has been registered.",
        type_name::<T>()
    )
}

#[async_trait]
impl<T, S> FromRequestParts<S> for TryInject<T>
where
    T: ?Sized + 'static,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(provider) = parts.extensions.get::<ServiceProvider>() {
            Ok(Self(provider.get::<T>()))
        } else {
            Ok(Self(None))
        }
    }
}

#[async_trait]
impl<T, S> FromRequestParts<S> for Inject<T>
where
    T: ?Sized + 'static,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(provider) = parts.extensions.get::<ServiceProvider>() {
            if let Some(service) = provider.get::<T>() {
                return Ok(Self(service));
            }
        }

        Err((StatusCode::INTERNAL_SERVER_ERROR, unregistered_type::<T>()))
    }
}

#[async_trait]
impl<T, S> FromRequestParts<S> for TryInjectMut<T>
where
    T: ?Sized + 'static,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(provider) = parts.extensions.get::<ServiceProvider>() {
            Ok(Self(provider.get_mut::<T>()))
        } else {
            Ok(Self(None))
        }
    }
}

#[async_trait]
impl<T, S> FromRequestParts<S> for InjectMut<T>
where
    T: ?Sized + 'static,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(provider) = parts.extensions.get::<ServiceProvider>() {
            if let Some(service) = provider.get_mut::<T>() {
                return Ok(Self(service));
            }
        }

        Err((StatusCode::INTERNAL_SERVER_ERROR, unregistered_type::<T>()))
    }
}

#[async_trait]
impl<T, S> FromRequestParts<S> for InjectAll<T>
where
    T: ?Sized + 'static,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(provider) = parts.extensions.get::<ServiceProvider>() {
            Ok(Self(provider.get_all::<T>().collect()))
        } else {
            Ok(Self(Vec::with_capacity(0)))
        }
    }
}

#[async_trait]
impl<T, S> FromRequestParts<S> for InjectAllMut<T>
where
    T: ?Sized + 'static,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(provider) = parts.extensions.get::<ServiceProvider>() {
            Ok(Self(provider.get_all_mut::<T>().collect()))
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
        extract::State,
        routing::{get, post},
        Router,
    };
    use di::{injectable, Injectable, ServiceCollection};
    use http::StatusCode;

    #[tokio::test]
    async fn request_should_fail_with_500_for_unregistered_service() {
        // arrange
        struct Service;

        impl Service {
            fn do_work(&self) -> String {
                "Test".into()
            }
        }

        async fn handler(Inject(service): Inject<Service>) -> String {
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
    async fn try_inject_into_handler() {
        // arrange
        #[injectable]
        struct Service;

        async fn handler(TryInject(_service): TryInject<Service>) -> StatusCode {
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
    async fn inject_into_handler() {
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

        async fn handler(Inject(service): Inject<dyn Service>) -> String {
            service.do_work()
        }

        let provider = ServiceCollection::new()
            .add(ServiceImpl::scoped())
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
    async fn inject_mutable_into_handler() {
        // arrange
        #[injectable]
        struct GlobalCounter(usize);

        #[injectable]
        struct SharedCounter(usize);

        #[injectable]
        struct SimpleCounter(usize);

        #[injectable]
        struct SharedCounterA {
            shared: RefMut<SharedCounter>,
        }

        #[injectable]
        struct SharedCounterB {
            shared: RefMut<SharedCounter>,
        }

        #[injectable]
        struct Counter {
            global: RefMut<GlobalCounter>,
            a: RefMut<SharedCounterA>,
            b: RefMut<SharedCounterB>,
            simple: RefMut<SimpleCounter>,
        }

        impl SharedCounterA {
            fn value(&self) -> usize {
                self.shared.read().unwrap().0
            }

            fn increment(&self) {
                self.shared.write().unwrap().0 += 1;
            }
        }

        impl SharedCounterB {
            fn value(&self) -> usize {
                self.shared.read().unwrap().0
            }

            fn increment(&self) {
                self.shared.write().unwrap().0 += 1;
            }
        }

        impl Counter {
            fn value(&self) -> usize {
                self.global.read().unwrap().0
                    + self.a.read().unwrap().value()
                    + self.b.read().unwrap().value()
                    + self.simple.read().unwrap().0
            }

            fn increment(&self) {
                self.global.write().unwrap().0 += 1;
                self.a.write().unwrap().increment();
                self.b.write().unwrap().increment();
                self.simple.write().unwrap().0 += 1;
            }
        }

        async fn handler(InjectMut(counter): InjectMut<Counter>) -> String {
            counter.write().unwrap().increment();
            counter.read().unwrap().value().to_string()
        }

        let provider = ServiceCollection::new()
            .add(GlobalCounter::singleton().as_mut())
            .add(SharedCounter::scoped().as_mut())
            .add(SharedCounterA::transient().as_mut())
            .add(SharedCounterB::transient().as_mut())
            .add(SimpleCounter::transient().as_mut())
            .add(Counter::transient().as_mut())
            .build_provider()
            .unwrap();

        let app = Router::new()
            .route("/count", get(handler))
            .with_provider(provider);

        let client = TestClient::new(app);

        // act
        let mut response = client.get("/count").send().await;

        // [Singleton] Global = 1
        // [Scoped] SharedCounterA = 1
        // [Scoped] SharedCounterB = 2
        // [Transient] SimpleCounter = 1
        //
        // 1 + 2 (shared) + 2 (shared) + 1 = 6
        let first = response.text().await;

        response = client.get("/count").send().await;

        // [Singleton] Global = 2
        // [Scoped] SharedCounterA = 1
        // [Scoped] SharedCounterB = 2
        // [Transient] SimpleCounter = 1
        //
        // 2 + 2 (shared) + 2 (shared) + 1 = 7
        let second = response.text().await;

        // assert
        assert_eq!(&first, "6");
        assert_eq!(&second, "7");
    }

    #[tokio::test]
    async fn inject_all_into_handler() {
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

        async fn handler(InjectAll(things): InjectAll<dyn Thing>) -> String {
            things.len().to_string()
        }

        let provider = ServiceCollection::new()
            .try_add_to_all(Thing1::scoped())
            .try_add_to_all(Thing2::scoped())
            .try_add_to_all(Thing3::scoped())
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
        assert_eq!(&text, "3");
    }

    #[tokio::test]
    async fn inject_with_state_into_handler() {
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

        #[derive(Clone)]
        struct AppState;

        async fn handler(
            Inject(service): Inject<dyn Service>,
            State(_state): State<AppState>,
        ) -> String {
            service.do_work()
        }

        let provider = ServiceCollection::new()
            .add(ServiceImpl::scoped())
            .build_provider()
            .unwrap();

        let app = Router::new()
            .route("/test", get(handler))
            .with_state(AppState)
            .with_provider(provider);

        let client = TestClient::new(app);

        // act
        let response = client.get("/test").send().await;
        let text = response.text().await;

        // assert
        assert_eq!(&text, "Test");
    }
}
