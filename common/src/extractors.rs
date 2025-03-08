cfg_if::cfg_if! { if #[cfg(feature = "ssr")] {
    use std::sync::Arc;
    use validator::Validate;
    use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
    use leptos::use_context;

    use crate::{Config, Error, Session, Store, Hub, Result, FnResult, FnError, FileManager};
}}

use crate::User;
use serde::{Deserialize, Serialize};

// ==================== // ArgsValidator // ==================== //

#[cfg(feature = "ssr")]
pub struct ArgsValidator;

#[cfg(feature = "ssr")]
impl ArgsValidator {
    pub fn validate<T: Validate>(arg: T) -> Result<T> {
        arg.validate()?;
        Ok(arg)
    }
}

// ==================== // HostExtractor // ==================== //

#[cfg(feature = "ssr")]
pub struct HostExtractor;

#[cfg(feature = "ssr")]
impl HostExtractor {
    pub async fn use_host() -> FnResult<String> {
        use axum::extract::Host;
        use leptos_axum::extract;

        let Host(host) = extract().await.map_err(|_| Error::InternalServer)?;
        Ok(format!("https://{}", host))
    }
}

// ==================== // StoreExtractor // ==================== //

#[cfg(feature = "ssr")]
pub struct StoreExtractor;

#[cfg(feature = "ssr")]
impl StoreExtractor {
    pub fn use_store() -> FnResult<Store> {
        use_context::<Store>()
            .ok_or_else(|| FnError::ServerError(String::from("failed to get store")))
    }
}

// ==================== // ConfigExtractor // ==================== //

#[cfg(feature = "ssr")]
pub struct ConfigExtractor;

#[cfg(feature = "ssr")]
impl ConfigExtractor {
    pub fn use_config() -> FnResult<Arc<Config>> {
        use_context::<Arc<Config>>()
            .ok_or_else(|| FnError::ServerError(String::from("failed to get config")))
    }
}

// ==================== // HubManager // ==================== //

#[cfg(feature = "ssr")]
pub struct HubManager;

#[derive(Deserialize, Serialize, Clone)]
pub struct FeedData {
    pub name: String,
    pub num_clients: i32,
    pub active_at: i64,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct HubData {
    pub num_feeds: i32,
    pub feeds: Vec<FeedData>,
    pub num_users: i32,
    pub users: Vec<User>,
    pub num_clients: i32,
    pub share_size: String,
}

#[cfg(feature = "ssr")]
impl HubManager {
    fn use_hub() -> FnResult<Hub> {
        use_context::<Hub>().ok_or_else(|| FnError::ServerError(String::from("failed to get hub")))
    }

    pub async fn get_data(store: &Store) -> FnResult<HubData> {
        let hub = Self::use_hub()?;
        let (num_feeds, feeds) = hub.get_feeds(5);
        let (num_clients, num_users, ids) = hub.get_users(5);

        let mut users = Vec::new();
        for user_id in ids {
            let user = User::get(user_id, &store).await?;
            users.push(user);
        }

        let config = ConfigExtractor::use_config()?;
        let share_size = FileManager::get_shared_size(config).await?;

        let rsp = HubData {
            num_feeds,
            feeds,
            num_users,
            users,
            num_clients,
            share_size,
        };
        Ok(rsp)
    }

    pub fn remove_user(user_id: i64) -> FnResult<()> {
        let hub = Self::use_hub()?;
        hub.remove(user_id);
        Ok(())
    }
}

// ==================== // CookieManager // ==================== //

pub struct CookieManager;

// Keys for cookie manager
#[cfg(feature = "ssr")]
const USER_ID_KEY: &str = "id";
#[cfg(feature = "ssr")]
const USER_SESSION_KEY: &str = "sess";
const DARK_MODE_KEY: &str = "darkmode";

impl CookieManager {
    /// Get auth data in the cookie
    ///
    #[cfg(feature = "ssr")]
    pub fn get_auth() -> FnResult<(i64, String)> {
        let cookie_jar = Self::use_cookie()?;
        let (user_id, session) = Self::extract_auth(cookie_jar)?;
        Ok((user_id, session))
    }

    /// Extract auth data from the cookie
    ///
    #[cfg(feature = "ssr")]
    pub fn extract_auth(cookie_jar: CookieJar) -> Result<(i64, String)> {
        let user_id = cookie_jar
            .get(USER_ID_KEY)
            .map(|cookie| cookie.value().parse::<i64>().ok())
            .flatten()
            .ok_or(Error::Unauthorized)?;

        let session = cookie_jar
            .get(USER_SESSION_KEY)
            .map(|cookie| cookie.value().to_owned())
            .ok_or(Error::Unauthorized)?;

        Ok((user_id, session))
    }

    /// Add auth data in the cookie
    ///
    #[cfg(feature = "ssr")]
    pub fn add_auth(user_id: i64, session: String) -> FnResult<()> {
        let cookie_jar = CookieJar::new();

        let cookie_jar = cookie_jar.add(
            Cookie::build((USER_ID_KEY, user_id.to_string()))
                .path("/")
                .same_site(SameSite::Strict)
                .secure(true)
                .http_only(true)
                .permanent(),
        );
        let cookie_jar = cookie_jar.add(
            Cookie::build((USER_SESSION_KEY, session))
                .path("/")
                .same_site(SameSite::Strict)
                .secure(true)
                .http_only(true)
                .permanent(),
        );
        Self::add_cookie_to_response(cookie_jar)
    }

    /// Remove auth data in the cookie
    ///
    #[cfg(feature = "ssr")]
    pub fn remove_auth() -> FnResult<(i64, String)> {
        let cookie_jar = CookieJar::new();

        let session = cookie_jar
            .get(USER_SESSION_KEY)
            .map(|cookie| cookie.value().to_owned())
            .unwrap_or_default();
        let user_id = cookie_jar
            .get(USER_ID_KEY)
            .map(|cookie| cookie.value().parse::<i64>().ok())
            .flatten()
            .unwrap_or_default();

        let cookie_jar = cookie_jar.add(Cookie::build((USER_ID_KEY, "")).path("/").removal());
        let cookie_jar = cookie_jar.add(Cookie::build((USER_SESSION_KEY, "")).path("/").removal());
        Self::add_cookie_to_response(cookie_jar)?;

        Ok((user_id, session))
    }

    /// Get darkmode in the cookie
    ///
    pub fn get_darkmode() -> bool {
        cfg_if::cfg_if! { if #[cfg(feature = "ssr")] {
            Self::use_cookie()
            .ok()
            .map(|cookie_jar| {
                cookie_jar
                    .get(DARK_MODE_KEY)
                    .map(|cookie| cookie.value().parse::<bool>().ok())
            })
            .flatten()
            .flatten()
            .unwrap_or(false)
        } else if #[cfg(feature = "hydrate")] {
            use leptos::*;
            use wasm_bindgen::JsCast;

            let doc = document().unchecked_into::<web_sys::HtmlDocument>();
            let cookie = doc.cookie().unwrap_or_default();
            let content = format!("{}=true", DARK_MODE_KEY);
            cookie.contains(&content)
        } else {
            false
        }}
    }

    /// Add darkmode in the cookie
    ///
    #[cfg(feature = "ssr")]
    pub fn add_darkmode(is_dark: bool) -> FnResult<()> {
        let cookie_jar = CookieJar::new();
        let cookie_jar = cookie_jar.add(
            Cookie::build((DARK_MODE_KEY, is_dark.to_string()))
                .path("/")
                .same_site(SameSite::Strict)
                .secure(true)
                .permanent(),
        );
        Self::add_cookie_to_response(cookie_jar)
    }

    /// Get cookie jar from context
    ///
    #[cfg(feature = "ssr")]
    fn use_cookie() -> FnResult<CookieJar> {
        use axum::http::request::Parts;

        let parts = use_context::<Parts>()
            .ok_or_else(|| FnError::ServerError(String::from("failed to get request parts")))?;

        let cookie_jar = CookieJar::from_headers(&parts.headers);
        Ok(cookie_jar)
    }

    /// Add cookies to the response
    ///
    #[cfg(feature = "ssr")]
    fn add_cookie_to_response(jar: CookieJar) -> FnResult<()> {
        use axum::http::header::SET_COOKIE;
        use leptos_axum::ResponseOptions;

        let response = use_context::<ResponseOptions>()
            .ok_or_else(|| FnError::ServerError(String::from("failed to get response")))?;

        for cookie in jar.iter() {
            if let Ok(value) = cookie.encoded().to_string().parse() {
                response.append_header(SET_COOKIE, value);
            }
        }
        Ok(())
    }
}

// ==================== // AuthExtractor // ==================== //

#[cfg(feature = "ssr")]
pub struct AuthExtractor;

#[cfg(feature = "ssr")]
impl AuthExtractor {
    pub async fn use_auth(refresh: bool, store: &Store) -> FnResult<User> {
        let (user_id, session) = CookieManager::get_auth()?;

        let user = Session::verify(user_id, session, refresh, &store).await?;
        Ok(user)
    }

    pub async fn use_admin(refresh: bool, store: &Store) -> FnResult<User> {
        let user = Self::use_auth(refresh, store).await?;
        if user.is_admin() {
            Ok(user)
        } else {
            Err(Error::Forbidden.into())
        }
    }
}
