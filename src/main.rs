use std::sync::Arc;
use std::thread;
use axum::{
    routing::{post, delete},
    Router,
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod avl;
mod idx;
mod handlers;
mod sst;

#[tokio::main()]
async fn main() {
    // Track AVL size thread
    let shared_state = Arc::new(avl::AVLTreeSingleton::new());

    thread::spawn({
        let shared_state = Arc::clone(&shared_state);
        move || {
            avl::check_size(shared_state);
        }
    });


    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create shared state

    // Build our application with a route
    let app = Router::new()
        .route("/set", post(handlers::set))
        .route("/get", post(handlers::get))
        .route("/delete", delete(handlers::delete))
        .with_state(shared_state)
        .layer(TraceLayer::new_for_http());

    // Run it with hyper on localhost:8000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}