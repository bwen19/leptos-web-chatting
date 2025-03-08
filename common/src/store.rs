use std::time::Duration;

use redis::aio::MultiplexedConnection;
use sqlx::{pool::PoolOptions, sqlite::SqlitePool};

use super::user::UserEntity;
use crate::{InsertUserArg, UserRole};

// ==================== // Store // ==================== //

#[derive(Clone)]
pub struct Store {
    pub pool: SqlitePool,
    pub con: MultiplexedConnection,
}

impl Store {
    /// Create a new store
    ///
    pub async fn new(config: &Config) -> Self {
        let pool = Store::create_database_pool(config).await;
        let con = Store::create_redis_connection(config).await;

        let store = Self { pool, con };
        store.init().await;

        store
    }

    /// Create a sqlite pool and run migration
    ///
    async fn create_database_pool(config: &Config) -> SqlitePool {
        let pool = PoolOptions::new()
            .max_connections(8)
            .connect(&config.db_url)
            .await
            .expect("failed to connect to database");

        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("failed to run migrate up");
        log::info!("database connected successfully");

        pool
    }

    /// Create a multiplexed tokio connection
    ///
    async fn create_redis_connection(config: &Config) -> MultiplexedConnection {
        let url = config.redis_url.clone();
        let client = redis::Client::open(url).expect("failed to create redis client");

        let mut con = client
            .get_multiplexed_tokio_connection_with_response_timeouts(
                Duration::from_secs(10),
                Duration::from_secs(10),
            )
            .await
            .expect("failed to get tokio connection");

        let _: String = redis::cmd("PING")
            .query_async(&mut con)
            .await
            .expect("failed to  connect to redis");
        log::info!("redis connected successfully");

        con
    }

    /// Create an account for admin if not exists
    ///
    async fn init(&self) {
        if let Ok(None) = UserEntity::find("admin", self).await {
            let user = InsertUserArg {
                username: String::from("admin"),
                password: String::from("123456"),
                role: UserRole::Admin,
                active: true,
            };
            user.insert(self)
                .await
                .expect("failed to create admin account");
            log::info!("admin account created");
        }
    }
}

// ==================== // Config // ==================== //

#[derive(Debug)]
pub struct Config {
    pub db_url: String,
    pub redis_url: String,
    pub site_root: String,
    pub avatar_dir: String,
    pub archive_dir: String,
    pub share_dir: String,
    pub expire_duration: Duration,
}

impl Config {
    /// Create config from env
    ///
    pub fn from_env() -> Self {
        let expire_days = env_default("CHAT_EXPIRE_DAYS", "3")
            .parse::<u64>()
            .expect("failed to parse expire days");

        Self {
            db_url: env_default("CHAT_DATABASE_URL", "sqlite://db/chat_dev.db"),
            redis_url: env_default("CHAT_REDIS_URL", "redis://:secret@localhost:6379/1"),
            site_root: env_default("LEPTOS_SITE_ROOT", "target/site"),
            avatar_dir: env_default("CHAT_AVATAR_DIR", "/assets/avatar"),
            archive_dir: env_default("CHAT_ARCHIVE_DIR", "/assets/archive"),
            share_dir: env_default("CHAT_SHARE_DIR", "/assets/share"),
            expire_duration: Duration::from_secs(expire_days * 60 * 60 * 24),
        }
    }
}

/// Helper for providing a default value
///
fn env_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or(default.to_string())
}
