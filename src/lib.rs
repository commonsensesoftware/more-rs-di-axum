#![doc = include_str!("../README.md")]

mod inject;
mod inject_keyed;

pub use inject::{Inject, InjectAll, InjectAllMut, InjectMut, TryInject, TryInjectMut};
pub use inject_keyed::{
    InjectAllWithKey, InjectAllWithKeyMut, InjectWithKey, InjectWithKeyMut, TryInjectWithKey, TryInjectWithKeyMut,
};

async fn services_middleware(
    State(provider): State<ServiceProvider>,
    mut request: Request,
    next: Next,
) -> Response {
    request.extensions_mut().insert(provider.create_scope());
    next.run(request).await
}

/// Provides [`axum::Router`] extension methods.
pub trait RouterServiceProviderExtensions {
    /// Adds the specified service provider to a router.
    ///
    /// # Arguments
    ///
    /// * `provider` - the [`di::ServiceProvider`] applied to the router
    ///
    /// # Remarks
    ///
    /// The service provider should be added after all routes are defined
    /// in the same manner as middleware.
    fn with_provider(self, provider: ServiceProvider) -> Self;
}

impl<S> RouterServiceProviderExtensions for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn with_provider(self, provider: ServiceProvider) -> Self {
        self.route_layer(from_fn_with_state(provider, services_middleware))
    }
}
