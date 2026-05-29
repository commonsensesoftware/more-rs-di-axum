#![doc = include_str!("../README.md")]

mod inject;
mod inject_keyed;

pub use inject::{Inject, InjectAll, InjectAllMut, InjectMut, TryInject, TryInjectMut};
pub use inject_keyed::{
    InjectAllWithKey, InjectAllWithKeyMut, InjectWithKey, InjectWithKeyMut, TryInjectWithKey, TryInjectWithKeyMut,
};

/// Contains library prelude.
pub mod prelude {
    use axum::{
        extract::{Request, State},
        middleware::{from_fn_with_state, Next},
        response::Response,
        Router,
    };
    use di::ServiceProvider;

    async fn services_middleware(
        State(provider): State<ServiceProvider>,
        mut request: Request,
        next: Next,
    ) -> Response {
        request.extensions_mut().insert(provider.create_scope());
        next.run(request).await
    }

    /// Provides [router][Router] extension methods.
    pub trait RouterExt: Sized {
        /// Adds the specified service provider to a router.
        ///
        /// # Arguments
        ///
        /// * `provider` - the [service provider][ServiceProvider] applied to the router
        ///
        /// # Remarks
        ///
        /// The service provider should be added after all routes are defined in the same manner as middleware.
        fn with_provider(self, provider: ServiceProvider) -> Self;
    }

    impl<S: Clone + Send + Sync + 'static> RouterExt for Router<S> {
        fn with_provider(self, provider: ServiceProvider) -> Self {
            self.route_layer(from_fn_with_state(provider, services_middleware))
        }
    }
}
