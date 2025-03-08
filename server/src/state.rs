use std::sync::Arc;

use axum::extract::FromRef;
use leptos::LeptosOptions;

use common::{Config, Hub, Store};

#[derive(Clone)]
pub struct AppState {
    pub leptos_options: Arc<LeptosOptions>,
    pub config: Arc<Config>,
    pub store: Store,
    pub hub: Hub,
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        (*state.leptos_options).clone()
    }
}

impl AppState {
    pub async fn new() -> Self {
        let leptos_options = leptos::get_configuration(None)
            .await
            .expect("failed to initialize Leptos Options")
            .leptos_options;

        let config = Config::from_env();
        let store = Store::new(&config).await;

        Self {
            leptos_options: Arc::new(leptos_options),
            config: Arc::new(config),
            store,
            hub: Hub::default(),
        }
    }
}
