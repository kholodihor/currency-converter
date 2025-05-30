mod routes;
mod templates;

use axum::Router;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

pub struct AppState {
    pub api_key: String,
}

pub async fn create_app(api_key: String) -> Router {
    let state = Arc::new(AppState { api_key });

    Router::new()
        .merge(routes::router(state))
        .layer(TraceLayer::new_for_http())
}
