#![doc = include_str!("README.md")]

mod inject;
mod inject_keyed;

use axum::{
    extract::{Request, State},
    middleware::{from_fn_with_state, Next},
    response::Response,
    Router,
};
use di::ServiceProvider;

pub use inject::*;
pub use inject_keyed::*;

#[cfg(test)]
mod test_client;

#[cfg(test)]
pub(crate) use test_client::*;

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

impl RouterServiceProviderExtensions for Router {
    fn with_provider(self, provider: ServiceProvider) -> Self {
        self.route_layer(from_fn_with_state(provider, services_middleware))
    }
}
