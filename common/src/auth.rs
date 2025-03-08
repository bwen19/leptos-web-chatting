cfg_if::cfg_if! { if #[cfg(feature = "ssr")] {
    use uuid::Uuid;
    use redis::AsyncCommands;
    use crate::{Result, Error, Store, User, DateTime};
    use super::user::{UserEntity};
}}

use serde::{Deserialize, Serialize};

// ==================== // LoginArg // ==================== //

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "ssr", derive(validator::Validate))]
pub struct LoginArg {
    #[cfg_attr(
        feature = "ssr",
        validate(length(min = 1, max = 128, message = "Username cannot be empty"))
    )]
    pub username: String,
    #[cfg_attr(
        feature = "ssr",
        validate(length(min = 6, max = 128, message = "Password is too short"))
    )]
    pub password: String,
}

impl LoginArg {
    /// Authenticate the user in the database
    ///
    #[cfg(feature = "ssr")]
    pub async fn call(&self, store: &Store) -> Result<(String, User)> {
        let user = UserEntity::find(&self.username, store)
            .await?
            .ok_or(Error::BadRequest(String::from("User not found")))?;

        user.verify_password(&self.password)?;

        let mut con = store.con.clone();

        let user = User::from(user);
        let ukey = user.key();
        let user_str = serde_json::to_string(&user)?;

        let skey = Session::make_key(user.id);
        let session = Uuid::new_v4().to_string();
        let now = DateTime::now().timestamp;

        let _: () = redis::pipe()
            .set_ex(ukey, user_str, 604800)
            .ignore()
            .zadd(&skey, &session, now)
            .ignore()
            .zremrangebyrank(skey, 0, -6)
            .query_async(&mut con)
            .await?;

        Ok((session, user))
    }
}

// ==================== // Session // ==================== //

#[derive(Deserialize, Serialize, Clone)]
pub struct Session {
    pub id: String,
    pub timestamp: i64,
    pub current: bool,
}

impl Session {
    /// Get a login user in the redis and database
    ///
    #[cfg(feature = "ssr")]
    pub async fn verify(
        user_id: i64,
        session: String,
        refresh: bool,
        store: &Store,
    ) -> Result<User> {
        let key = Session::make_key(user_id);
        let mut con = store.con.clone();

        let score: Option<i64> = con.zscore(&key, &session).await?;
        if score.is_none() {
            return Err(Error::Unauthorized);
        }

        let user = User::get(user_id, store)
            .await
            .map_err(|_| Error::BadRequest(String::from("User not exists")))?;

        if !user.active {
            return Err(Error::Forbidden);
        }

        if refresh {
            let now = DateTime::now().timestamp;
            let _: () = con.zadd(key, session, now).await?;
        }

        Ok(user)
    }

    /// List all sessions of the user in the redis
    ///
    #[cfg(feature = "ssr")]
    pub async fn list(user_id: i64, session: String, store: &Store) -> Result<Vec<Self>> {
        let key = Self::make_key(user_id);
        let mut con = store.con.clone();

        let data: Vec<(String, i64)> = con.zrange_withscores(key, 0, -1).await?;
        let ret: Vec<Self> = data
            .into_iter()
            .map(|(id, timestamp)| Self {
                current: id == session,
                id,
                timestamp,
            })
            .collect();

        if ret.iter().any(|x| x.current) {
            Ok(ret)
        } else {
            Err(Error::Unauthorized)
        }
    }

    /// Delete the login session from redis
    ///
    #[cfg(feature = "ssr")]
    pub async fn delete(user_id: i64, session: String, store: &Store) -> Result<()> {
        let key = Self::make_key(user_id);
        let mut con = store.con.clone();
        let _: () = con.zrem(key, session).await?;
        Ok(())
    }

    /// Create a key of session from user_id for redis
    ///
    #[cfg(feature = "ssr")]
    fn make_key(user_id: i64) -> String {
        format!("session:{}", user_id)
    }
}
