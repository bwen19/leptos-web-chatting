use std::time::Duration;
use tokio::{net::TcpListener, signal};

use axum::{error_handling::HandleErrorLayer, http::StatusCode, routing::get, BoxError, Router};
use leptos::provide_context;
use leptos_axum::LeptosRoutes;
use tower::{timeout::TimeoutLayer, ServiceBuilder};

use app::App;
use ws::ws_handler;

mod fallback;
mod state;
mod ws;

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).expect("failed to initialize logging");

    let app_state = state::AppState::new().await;
    let addr = app_state.leptos_options.site_addr;
    let routes = leptos_axum::generate_route_list(App);

    // build our application with a route
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                let config = app_state.config.clone();
                let store = app_state.store.clone();
                let hub = app_state.hub.clone();
                move || {
                    provide_context(config.clone());
                    provide_context(store.clone());
                    provide_context(hub.clone());
                }
            },
            App,
        )
        .fallback(fallback::file_and_error_handler)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|_: BoxError| async {
                    StatusCode::REQUEST_TIMEOUT
                }))
                .layer(TimeoutLayer::new(Duration::from_secs(10))),
        )
        .with_state(app_state);

    log::info!("listening on http://{}", &addr);
    let listener = TcpListener::bind(addr)
        .await
        .expect("failed to create tcp listener");

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("failed to start the http server");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    log::info!("starting graceful shutdown...");
}
