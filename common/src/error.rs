cfg_if::cfg_if! { if #[cfg(feature = "ssr")] {
    use axum::{
        http::StatusCode,
        response::{IntoResponse, Response},
    };
}}

use leptos::ServerFnError;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub type FnError = ServerFnError<Error>;
pub type FnResult<T> = core::result::Result<T, ServerFnError<Error>>;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(thiserror::Error, Serialize, Deserialize, Clone, Debug)]
pub enum Error {
    #[error("{0}")]
    BadRequest(String), // 400

    #[error("Unauthorized")]
    Unauthorized, // 401

    #[error("Forbidden")]
    Forbidden, // 403

    #[error("Not Found")]
    NotFound, // 404

    #[error("Internal Server Error")]
    InternalServer, // 500

    #[error("The channel is disconnected")]
    SendError, // 500
}

impl FromStr for Error {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Unauthorized" => Ok(Self::Unauthorized),
            "Forbidden" => Ok(Self::Forbidden),
            "Not Found" => Ok(Self::NotFound),
            "Internal Server Error" => Ok(Self::InternalServer),
            _ => Ok(Self::BadRequest(s.to_string())),
        }
    }
}

#[cfg(feature = "ssr")]
impl Error {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Error::BadRequest(_) => StatusCode::BAD_REQUEST,
            Error::Unauthorized => StatusCode::UNAUTHORIZED,
            Error::Forbidden => StatusCode::FORBIDDEN,
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::InternalServer | Error::SendError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(feature = "ssr")]
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (self.status_code(), self.to_string()).into_response()
    }
}

#[cfg(feature = "ssr")]
impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::Database(dbe) => {
                if dbe.is_unique_violation() {
                    Self::BadRequest(String::from("Username already exists"))
                } else {
                    log::error!("database: {}", dbe);
                    Self::InternalServer
                }
            }
            sqlx::Error::RowNotFound => Self::NotFound,
            _ => {
                log::error!("{}", err);
                Self::InternalServer
            }
        }
    }
}

#[cfg(feature = "ssr")]
impl From<validator::ValidationErrors> for Error {
    fn from(errs: validator::ValidationErrors) -> Self {
        if let Some((field, errs)) = errs.field_errors().iter().next() {
            if let Some(err) = errs.first() {
                let s = if let Some(msg) = err.message.as_ref() {
                    msg.to_string()
                } else {
                    format!("validation error of {} for field {}", err.code, field)
                };
                return Self::BadRequest(s);
            }
        }
        log::error!("validator: something unexpected happened");
        Self::InternalServer
    }
}

#[cfg(feature = "ssr")]
impl From<redis::RedisError> for Error {
    fn from(err: redis::RedisError) -> Self {
        log::error!("redis: {}", err);
        Self::InternalServer
    }
}

#[cfg(feature = "ssr")]
impl From<argon2::Error> for Error {
    fn from(err: argon2::Error) -> Self {
        log::error!("argon2: {}", err);
        Self::InternalServer
    }
}

#[cfg(feature = "ssr")]
impl From<serde_json::error::Error> for Error {
    fn from(err: serde_json::error::Error) -> Self {
        log::error!("serde json: {}", err);
        Self::InternalServer
    }
}

#[cfg(feature = "ssr")]
impl From<std::convert::Infallible> for Error {
    fn from(_: std::convert::Infallible) -> Self {
        log::error!("infallible: something unexpected happened");
        Self::InternalServer
    }
}

#[cfg(feature = "ssr")]
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        log::error!("io error: {}", err);
        Self::InternalServer
    }
}

#[cfg(feature = "ssr")]
impl<T> From<tokio::sync::broadcast::error::SendError<T>> for Error {
    fn from(err: tokio::sync::broadcast::error::SendError<T>) -> Self {
        log::error!("tokio broadcast: {}", err);
        Self::SendError
    }
}

#[cfg(feature = "ssr")]
impl From<ServerFnError> for Error {
    fn from(err: ServerFnError) -> Self {
        log::error!("server fn error: {}", err);
        Self::InternalServer
    }
}
