use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::{
    async_trait,
    body::Body,
    http::{Request, Response, StatusCode, Uri},
    response::{IntoResponse, Response as AxumResponse},
};
use axum_extra::extract::cookie::CookieJar;
use leptos::LeptosOptions;
use tower::ServiceExt;
use tower_http::services::ServeDir;

use crate::state::AppState;
use app::App;
use common::{CookieManager, Error, Session};

// ==================== // ShareGuard // ==================== //

pub struct ShareGuard;

#[async_trait]
impl FromRequestParts<AppState> for ShareGuard {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let uri = Uri::from_request_parts(parts, state).await?;
        if uri.path().starts_with(&state.config.share_dir) {
            let cookie_jar = CookieJar::from_request_parts(parts, state).await?;
            let (user_id, session) = CookieManager::extract_auth(cookie_jar)?;

            let _ = Session::verify(user_id, session, false, &state.store).await?;
        }
        Ok(ShareGuard)
    }
}

// ==================== // file_and_error_handler // ==================== //

pub async fn file_and_error_handler(
    uri: Uri,
    _share: ShareGuard,
    State(options): State<LeptosOptions>,
    req: Request<Body>,
) -> AxumResponse {
    let root = options.site_root.clone();
    let res = get_static_file(uri.clone(), &root).await.unwrap();

    if res.status() == StatusCode::OK {
        res.into_response()
    } else {
        let handler = leptos_axum::render_app_to_stream(options.to_owned(), App);
        handler(req).await.into_response()
    }
}

async fn get_static_file(uri: Uri, root: &str) -> Result<Response<Body>, (StatusCode, String)> {
    let req = Request::builder()
        .uri(uri.clone())
        .body(Body::empty())
        .unwrap();
    // `ServeDir` implements `tower::Service` so we can call it with `tower::ServiceExt::oneshot`
    // This path is relative to the cargo root
    match ServeDir::new(root).oneshot(req).await {
        Ok(res) => Ok(res.map(Body::new)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {err}"),
        )),
    }
}
