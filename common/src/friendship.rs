cfg_if::cfg_if! { if #[cfg(feature = "ssr")] {
    use crate::{Error, Result, Store, Room, User};
}}

use serde::{Deserialize, Serialize};

// ==================== // Friend // ==================== //

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
#[repr(u8)]
pub enum FriendStatus {
    Accepted = 1,
    Adding = 2,
    Added = 3,
    Deleted = 4,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Friend {
    pub id: i64,
    pub username: String,
    pub nickname: String,
    pub avatar: String,
    pub status: FriendStatus,
    pub room_id: String,
}

impl Eq for Friend {}

impl PartialEq for Friend {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Friend {
    /// Get friend from the friendship
    ///
    #[cfg(feature = "ssr")]
    pub async fn get(user_id: i64, fsp: &FriendShip, store: &Store) -> Result<(Friend, Friend)> {
        let user0 = User::get(fsp.id0, store).await?;
        let user1 = User::get(fsp.id1, store).await?;
        let friend0 = Friend::from_user(fsp, user0, true);
        let friend1 = Friend::from_user(fsp, user1, false);

        if user_id == friend0.id {
            Ok((friend0, friend1))
        } else {
            Ok((friend1, friend0))
        }
    }

    /// Get all friends of a user
    ///
    #[cfg(feature = "ssr")]
    pub async fn get_all(user_id: i64, store: &Store) -> Result<Vec<Friend>> {
        let rows: Vec<FriendInfoRow> = sqlx::query_as(
            "SELECT * FROM friendships AS f JOIN users AS u ON u.id = f.id0 WHERE f.id1 = $1 AND status != $2",
        )
        .bind(user_id)
        .bind(FriendStatus::Deleted)
        .fetch_all(&store.pool)
        .await?;

        let mut friends0: Vec<Friend> = rows
            .into_iter()
            .filter(|row| row.friendship.status != FriendStatus::Deleted)
            .map(|row| Self::from_user(&row.friendship, row.user, true))
            .collect();

        let rows: Vec<FriendInfoRow> = sqlx::query_as(
            "SELECT * FROM friendships AS f JOIN users AS u ON u.id = f.id1 WHERE f.id0 = $1 AND status != $2",
        )
        .bind(user_id)
        .bind(FriendStatus::Deleted)
        .fetch_all(&store.pool)
        .await?;

        let mut friends1 = rows
            .into_iter()
            .filter(|row| row.friendship.status != FriendStatus::Deleted)
            .map(|row| Self::from_user(&row.friendship, row.user, false))
            .collect();

        friends0.append(&mut friends1);
        Ok(friends0)
    }

    /// Convert Friendship and User into Friend
    ///
    #[cfg(feature = "ssr")]
    fn from_user(fsp: &FriendShip, user: User, first: bool) -> Self {
        Self {
            id: user.id,
            username: user.username,
            nickname: user.nickname,
            avatar: user.avatar,
            status: fsp.status(first),
            room_id: Room::friend_room_id(fsp),
        }
    }
}

#[cfg(feature = "ssr")]
#[derive(sqlx::FromRow)]
struct FriendInfoRow {
    #[sqlx(flatten)]
    friendship: FriendShip,
    #[sqlx(flatten)]
    user: User,
}

// ==================== // FriendShip // ==================== //

#[cfg(feature = "ssr")]
#[derive(Clone, Copy, sqlx::FromRow)]
pub struct FriendShip {
    pub id0: i64,
    pub id1: i64,
    pub status: FriendStatus,
}

#[cfg(feature = "ssr")]
impl FriendShip {
    /// Create a friendship in the database
    ///
    pub async fn add(user_id: i64, friend_id: i64, store: &Store) -> Result<Self> {
        if let Some(friendship) = Self::find(user_id, friend_id, store).await? {
            return match friendship.status {
                FriendStatus::Deleted => {
                    let status = if friendship.id0 == user_id {
                        FriendStatus::Adding
                    } else {
                        FriendStatus::Added
                    };
                    friendship.update(status, store).await
                }
                _ => Err(Error::BadRequest(String::from("Status must be deleted"))),
            };
        }

        let row: Self = sqlx::query_as(
            "
            INSERT INTO friendships (id0, id1, status)
            VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(&user_id)
        .bind(&friend_id)
        .bind(FriendStatus::Adding)
        .fetch_one(&store.pool)
        .await?;

        Ok(row)
    }

    /// Change the friendship's status to Accepted
    ///
    pub async fn accept(user_id: i64, friend_id: i64, store: &Store) -> Result<Self> {
        if let Some(friendship) = Self::find(user_id, friend_id, store).await? {
            if (user_id == friendship.id0 && friendship.status != FriendStatus::Added)
                || (user_id == friendship.id1 && friendship.status != FriendStatus::Adding)
            {
                return Err(Error::BadRequest(String::from("Status must be added")));
            }
            return friendship.update(FriendStatus::Accepted, store).await;
        }
        Err(Error::NotFound)
    }

    /// Revert a friendship by changing the status to Delete
    ///
    pub async fn revert(user_id: i64, friend_id: i64, store: &Store) -> Result<Self> {
        if let Some(friendship) = Self::find(user_id, friend_id, store).await? {
            if friendship.status != FriendStatus::Added && friendship.status != FriendStatus::Adding
            {
                return Err(Error::BadRequest(String::from(
                    "Status must be adding or added",
                )));
            }
            return friendship.update(FriendStatus::Deleted, store).await;
        }
        Err(Error::NotFound)
    }

    /// Delete a friendship by changing the status to Delete
    ///
    pub async fn delete(user_id: i64, friend_id: i64, store: &Store) -> Result<Self> {
        if let Some(friendship) = Self::find(user_id, friend_id, store).await? {
            if friendship.status != FriendStatus::Accepted {
                return Err(Error::BadRequest(String::from("Status must be accepted")));
            }
            return friendship.update(FriendStatus::Deleted, store).await;
        }
        Err(Error::NotFound)
    }

    /// Find the friendship between user and friend
    ///
    async fn find(user_id: i64, friend_id: i64, store: &Store) -> Result<Option<Self>> {
        let row: Option<Self> = sqlx::query_as(
            "
            SELECT * FROM friendships
            WHERE (id0 = $1 AND id1 = $2) OR (id1 = $1 AND id0 = $2)",
        )
        .bind(&user_id)
        .bind(&friend_id)
        .fetch_optional(&store.pool)
        .await?;

        Ok(row)
    }

    /// Update the status of a friendship
    ///
    async fn update(&self, status: FriendStatus, store: &Store) -> Result<Self> {
        let row: Self = sqlx::query_as(
            "
            UPDATE friendships SET status = $1
            WHERE id0 = $2 AND id1 = $3 RETURNING *",
        )
        .bind(&status)
        .bind(&self.id0)
        .bind(&self.id1)
        .fetch_one(&store.pool)
        .await?;
        Ok(row)
    }

    /// Get the friend status by the position of two friends
    ///
    fn status(&self, first: bool) -> FriendStatus {
        if first {
            self.status
        } else {
            match self.status {
                FriendStatus::Adding => FriendStatus::Added,
                FriendStatus::Added => FriendStatus::Adding,
                _ => self.status,
            }
        }
    }
}
