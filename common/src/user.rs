cfg_if::cfg_if! { if #[cfg(feature = "ssr")] {
    use rand::Rng;
    use redis::AsyncCommands;
    use crate::{Error, Result, Store};
}}

use serde::{Deserialize, Serialize};

// ==================== // User // ==================== //

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
#[repr(u8)]
pub enum UserRole {
    Admin = 1,
    User = 2,
}

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct User {
    pub id: i64,
    pub username: String,
    pub nickname: String,
    pub avatar: String,
    pub role: UserRole,
    pub active: bool,
}

impl User {
    /// Get a user by id in the redis or database
    ///
    #[cfg(feature = "ssr")]
    pub async fn get(user_id: i64, store: &Store) -> Result<Self> {
        let key = Self::make_key(user_id);
        let mut con = store.con.clone();

        let user_str: Option<String> = con.get(&key).await?;
        if let Some(user_str) = user_str {
            let user = serde_json::from_str::<Self>(&user_str)?;
            return Ok(user);
        }

        let user: User = sqlx::query_as(
            "
            SELECT id, username, nickname, avatar, role, active FROM users
            WHERE id = $1 AND active = 1",
        )
        .bind(&user_id)
        .fetch_one(&store.pool)
        .await?;

        let user_str = serde_json::to_string(&user)?;
        let _: () = con.set_ex(&key, user_str, 604800).await?;
        Ok(user)
    }

    /// Find a user by username or nickname in the database
    ///
    #[cfg(feature = "ssr")]
    pub async fn find(keyword: &str, store: &Store) -> Result<Option<Self>> {
        let user: Option<Self> = sqlx::query_as(
            "
            SELECT id, username, nickname, avatar, role, active FROM users
            WHERE username = $1 OR nickname = $1",
        )
        .bind(&keyword)
        .fetch_optional(&store.pool)
        .await?;

        match user {
            Some(u) if u.active => Ok(Some(u)),
            _ => Ok(None),
        }
    }

    /// Delete a user by id in the database
    ///
    #[cfg(feature = "ssr")]
    pub async fn delete(user_id: i64, store: &Store) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(&user_id)
            .execute(&store.pool)
            .await?;

        let key = Self::make_key(user_id);
        let mut con = store.con.clone();
        let _: () = con.del(key).await?;
        Ok(())
    }

    /// Returns whether the user is an administrator
    ///
    pub fn is_admin(&self) -> bool {
        match self.role {
            UserRole::Admin => true,
            _ => false,
        }
    }

    /// Returns a key of user for redis
    ///
    #[cfg(feature = "ssr")]
    pub fn key(&self) -> String {
        Self::make_key(self.id)
    }

    /// Create a key from user_id for redis
    ///
    #[cfg(feature = "ssr")]
    pub fn make_key(user_id: i64) -> String {
        format!("user:{}", user_id)
    }
}

// ==================== // ListUsers // ==================== //

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ssr", derive(validator::Validate))]
pub struct ListUsersArg {
    #[cfg_attr(
        feature = "ssr",
        validate(range(min = 1, message = "Page id must be greater than 0"))
    )]
    pub page_id: i32,
    #[cfg_attr(
        feature = "ssr",
        validate(range(min = 5, max = 25, message = "Page size must be between 5 and 25"))
    )]
    pub page_size: i32,
    pub role: Option<UserRole>,
}

impl Default for ListUsersArg {
    fn default() -> Self {
        Self {
            page_id: 1,
            page_size: 8,
            role: None,
        }
    }
}

impl ListUsersArg {
    /// Get a list of users in the database
    ///
    #[cfg(feature = "ssr")]
    pub async fn call(&self, store: &Store) -> Result<ListUsersRsp> {
        let offset = (self.page_id - 1) * self.page_size;

        let result: Vec<ListUsersRow> = sqlx::query_as(
            "
            SELECT id, username, nickname, avatar, role, active, count(*) OVER() AS total
            FROM users WHERE role = coalesce($1, role) LIMIT $2 OFFSET $3",
        )
        .bind(self.role)
        .bind(self.page_size)
        .bind(offset)
        .fetch_all(&store.pool)
        .await?;

        let total = match result.get(0) {
            Some(row) => row.total.unwrap_or(0),
            None => 0,
        };
        let users: Vec<User> = result.into_iter().map(|row| row.into()).collect();

        Ok(ListUsersRsp { total, users })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ListUsersRsp {
    pub total: i32,
    pub users: Vec<User>,
}

#[cfg(feature = "ssr")]
#[derive(sqlx::FromRow)]
struct ListUsersRow {
    id: i64,
    username: String,
    nickname: String,
    avatar: String,
    role: UserRole,
    active: bool,
    total: Option<i32>,
}

#[cfg(feature = "ssr")]
impl From<ListUsersRow> for User {
    fn from(v: ListUsersRow) -> Self {
        Self {
            id: v.id,
            username: v.username,
            nickname: v.nickname,
            avatar: v.avatar,
            role: v.role,
            active: v.active,
        }
    }
}

// ==================== // InsertUserArg // ==================== //

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "ssr", derive(validator::Validate))]
pub struct InsertUserArg {
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
    pub role: UserRole,
    pub active: bool,
}

impl InsertUserArg {
    /// Insert a new user in the database
    ///
    #[cfg(feature = "ssr")]
    pub async fn insert(&self, store: &Store) -> Result<()> {
        let hash = hash_password(&self.password)?;

        sqlx::query(
            "
            INSERT INTO users (username, password, nickname, avatar, role, active)
            VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&self.username)
        .bind(&hash)
        .bind(&self.username)
        .bind("")
        .bind(&self.role)
        .bind(&self.active)
        .execute(&store.pool)
        .await?;

        Ok(())
    }
}

// ==================== // UpdateUserArg // ==================== //

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "ssr", derive(validator::Validate))]
pub struct UpdateUserArg {
    #[cfg_attr(feature = "ssr", validate(range(min = 1, message = "Invalid user id")))]
    pub id: i64,
    #[cfg_attr(
        feature = "ssr",
        validate(length(min = 1, max = 128, message = "Username cannot be empty"))
    )]
    pub username: Option<String>,
    #[cfg_attr(
        feature = "ssr",
        validate(length(min = 6, max = 128, message = "Password is too short"))
    )]
    pub password: Option<String>,
    #[cfg_attr(
        feature = "ssr",
        validate(length(min = 1, max = 128, message = "Nickname cannot be empty"))
    )]
    pub nickname: Option<String>,
    #[cfg_attr(
        feature = "ssr",
        validate(length(min = 1, max = 256, message = "Avatar cannot be empty"))
    )]
    pub avatar: Option<String>,
    pub role: Option<UserRole>,
    pub active: Option<bool>,
}

impl UpdateUserArg {
    /// Update the user in the database
    ///
    #[cfg(feature = "ssr")]
    pub async fn call(&self, store: &Store) -> Result<User> {
        let hash: Option<String> = match self.password {
            Some(ref value) => {
                let hash = hash_password(value)?;
                Some(hash)
            }
            None => None,
        };

        let user: User = sqlx::query_as(
            "
            UPDATE users
            SET
                username = coalesce($1, username),
                password = coalesce($2, password),
                nickname = coalesce($3, nickname),
                avatar = coalesce($4, avatar),
                role = coalesce($5, role),
                active = coalesce($6, active)
            WHERE id = $7
            RETURNING *",
        )
        .bind(&self.username)
        .bind(&hash)
        .bind(&self.nickname)
        .bind(&self.avatar)
        .bind(&self.role)
        .bind(&self.active)
        .bind(&self.id)
        .fetch_one(&store.pool)
        .await?;

        let mut con = store.con.clone();
        let key = user.key();
        let user_str = serde_json::to_string(&user)?;
        let _: () = con.set_ex(&key, user_str, 604800).await?;
        Ok(user)
    }
}

// ==================== // UpdatePasswordArg // ==================== //

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "ssr", derive(validator::Validate))]
pub struct UpdatePasswordArg {
    #[cfg_attr(feature = "ssr", validate(range(min = 1, message = "Invalid user id")))]
    pub id: i64,
    #[cfg_attr(
        feature = "ssr",
        validate(length(min = 6, max = 128, message = "Old password is too short"))
    )]
    pub old_password: String,
    #[cfg_attr(
        feature = "ssr",
        validate(length(min = 6, max = 128, message = "New password is too short"))
    )]
    pub new_password: String,
    #[cfg_attr(
        feature = "ssr",
        validate(length(min = 6, max = 128, message = "Confirm password is too short"))
    )]
    pub confirm_password: String,
}

impl UpdatePasswordArg {
    /// Update the password in the database
    ///
    #[cfg(feature = "ssr")]
    pub async fn call(&self, store: &Store) -> Result<()> {
        if self.new_password != self.confirm_password {
            return Err(Error::BadRequest(String::from(
                "The password confirmation does not match.",
            )));
        }

        let hash: String = sqlx::query_scalar("SELECT password FROM users WHERE id = $1")
            .bind(self.id)
            .fetch_one(&store.pool)
            .await?;
        verify_password(&hash, &self.old_password)?;

        let password = hash_password(&self.new_password)?;
        sqlx::query("UPDATE users SET password = $1 WHERE id = $2")
            .bind(password)
            .bind(self.id)
            .execute(&store.pool)
            .await?;

        Ok(())
    }
}

// ==================== // UserEntity // ==================== //

#[cfg(feature = "ssr")]
#[derive(sqlx::FromRow)]
pub struct UserEntity {
    id: i64,
    username: String,
    password: String,
    nickname: String,
    avatar: String,
    role: UserRole,
    active: bool,
}

#[cfg(feature = "ssr")]
impl UserEntity {
    /// Get a user by username
    ///
    pub async fn find(username: &str, store: &Store) -> Result<Option<Self>> {
        let user: Option<Self> = sqlx::query_as("SELECT * FROM users WHERE username = $1")
            .bind(&username)
            .fetch_optional(&store.pool)
            .await?;
        match user {
            Some(u) if u.active => Ok(Some(u)),
            _ => Ok(None),
        }
    }

    /// Verifies the password with the encoded hash
    ///
    pub fn verify_password(&self, password: &str) -> Result<()> {
        verify_password(&self.password, password)
    }
}

#[cfg(feature = "ssr")]
impl From<UserEntity> for User {
    fn from(val: UserEntity) -> Self {
        Self {
            id: val.id,
            username: val.username,
            nickname: val.nickname,
            avatar: val.avatar,
            role: val.role,
            active: val.active,
        }
    }
}

// ==================== // UTILS // ==================== //

/// Returns the hash of a password string
///
#[cfg(feature = "ssr")]
fn hash_password(pwd: &str) -> Result<String> {
    let salt: [u8; 32] = rand::thread_rng().gen();
    let config = argon2::Config::rfc9106_low_mem();
    let hash = argon2::hash_encoded(pwd.as_bytes(), &salt, &config)?;
    Ok(hash)
}

/// Verifies the password with the encoded hash
///
#[cfg(feature = "ssr")]
fn verify_password(hash: &str, password: &str) -> Result<()> {
    if !argon2::verify_encoded(hash, password.as_bytes())? {
        return Err(Error::BadRequest(String::from("The password is incorrect")));
    }
    Ok(())
}
