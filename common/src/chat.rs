cfg_if::cfg_if! { if #[cfg(feature = "ssr")] {
    use redis::{AsyncCommands, aio::MultiplexedConnection};
    use crate::{Result, Error, Store, FriendShip, FriendStatus};
}}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{DateTime, FileMeta, Friend, User};

// ==================== // Event // ==================== //

#[derive(Deserialize, Serialize)]
pub enum Event {
    // initial data from server
    InitRooms(Vec<Room>),
    InitFriends(Vec<Friend>),
    InitMessages(HashMap<String, Vec<Message>>),
    // handle message
    Send(Message),
    Receive(Message),
    // handle friendship
    AddFriend(i64),
    AcceptFriend(i64),
    RevertFriend(i64),
    DeleteFriend(i64),
    ReceiveFriend(Friend),
    ReceiveRoom(Room),
    // handle call
    SendCall(i64),
    SendCallDone(i64),
    ReceiveCall(i64, Uuid),
    SendReply(i64, Uuid),
    ReceiveReply(Uuid),
    SendHungUp(i64, HungUpReson),
    ReceiveHungUp(HungUpReson),
    SendOffer(i64, Uuid, String),
    ReceiveOffer(String),
    SendAnswer(i64, Uuid, String),
    ReceiveAnswer(String),
    SendCandidate(i64, Uuid, IceCandidate),
    ReceiveCandidate(IceCandidate),
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[repr(u8)]
pub enum HungUpReson {
    Offline = 1,
    Busy = 2,
    Refuse = 3,
    Cancel = 4,
    Finish = 5,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IceCandidate {
    pub candidate: String,
    pub sdp_mid: String,
    pub sdp_m_line_index: u16,
}

// ==================== // Message // ==================== //

#[derive(Serialize, Deserialize, Clone, Copy)]
#[repr(u8)]
pub enum MessageKind {
    Text = 1,
    Image = 2,
    File = 3,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: Uuid,
    pub content: String,
    pub url: String,
    pub kind: MessageKind,
    pub divide: bool,
    pub room_id: String,
    pub sender: User,
    pub send_at: i64,
}

impl Message {
    /// Create a new text message
    ///
    pub fn text(room_id: String, sender: User, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            content,
            url: String::new(),
            kind: MessageKind::Text,
            divide: false,
            room_id,
            sender,
            send_at: DateTime::now().timestamp,
        }
    }

    /// Create a new file message
    ///
    pub fn file(room_id: String, sender: User, file_meta: FileMeta) -> Self {
        let kind = if file_meta.img {
            MessageKind::Image
        } else {
            MessageKind::File
        };

        Self {
            id: Uuid::new_v4(),
            content: file_meta.name,
            url: file_meta.url,
            kind,
            divide: false,
            room_id,
            sender,
            send_at: DateTime::now().timestamp,
        }
    }

    /// Update divide by last send time
    ///
    #[cfg(feature = "ssr")]
    pub fn update(self, last_send_at: i64) -> Self {
        let divide = (self.send_at - last_send_at) > 400;

        Self {
            id: self.id,
            content: self.content,
            url: self.url,
            kind: self.kind,
            divide,
            room_id: self.room_id,
            sender: self.sender,
            send_at: self.send_at,
        }
    }

    /// Store the message in the redis
    ///
    #[cfg(feature = "ssr")]
    pub async fn cache(&self, store: &Store) -> Result<()> {
        let value = serde_json::to_string(&self)?;
        let mut con = store.con.clone();
        let _: () = redis::pipe()
            .lpush(&self.room_id, value)
            .ignore()
            .ltrim(&self.room_id, 0, 35)
            .query_async(&mut con)
            .await?;

        Ok(())
    }

    /// Get a list of messages from redis
    ///
    #[cfg(feature = "ssr")]
    pub async fn list(room_id: &String, con: &mut MultiplexedConnection) -> Result<Vec<Self>> {
        let messages: Vec<String> = con.lrange(room_id, 0, 35).await?;
        messages
            .into_iter()
            .rev()
            .map(|s| serde_json::from_str::<Self>(&s).map_err(|_| Error::InternalServer))
            .collect()
    }
}

// ==================== // Room // ==================== //

#[derive(Serialize, Deserialize, Clone)]
pub struct Room {
    pub key: Uuid,
    pub id: String,
    pub name: String,
    pub cover: String,
    pub unreads: u32,
    pub content: String,
    pub send_at: i64,
}

impl Room {
    /// Update the room info by the latest message
    ///
    pub fn update(self, message: &Message, incr: bool) -> Self {
        let num = if incr { 1_u32 } else { 0_u32 };
        Self {
            key: Uuid::new_v4(),
            id: self.id,
            name: self.name,
            cover: self.cover,
            unreads: self.unreads + num,
            content: message.content.clone(),
            send_at: message.send_at,
        }
    }

    /// Change the unreads to zero and update key
    ///
    pub fn check(&mut self) {
        self.unreads = 0;
        self.key = Uuid::new_v4();
    }

    /// Get user and friend rooms frome the friendship
    ///
    #[cfg(feature = "ssr")]
    pub async fn get(user_id: i64, fsp: &FriendShip, store: &Store) -> Result<(Room, Room)> {
        let (user, friend) = Friend::get(user_id, fsp, store).await?;
        Ok((user.into(), friend.into()))
    }

    /// Get room id of the user's room
    ///
    #[cfg(feature = "ssr")]
    pub fn user_room_id(user_id: i64) -> String {
        format!("chats:private-{}", user_id)
    }

    /// Get room id of the friendship's room
    ///
    #[cfg(feature = "ssr")]
    pub fn friend_room_id(fsp: &FriendShip) -> String {
        format!("chats:room-{}-{}", fsp.id0, fsp.id1)
    }
}

#[cfg(feature = "ssr")]
impl From<Friend> for Room {
    fn from(friend: Friend) -> Self {
        Self {
            key: Uuid::new_v4(),
            id: friend.room_id,
            name: friend.nickname,
            cover: friend.avatar,
            unreads: 0,
            content: String::new(),
            send_at: 0,
        }
    }
}

// ==================== // Chats // ==================== //

#[cfg(feature = "ssr")]
pub struct Chats {
    pub rooms: Vec<Room>,
    pub friends: Vec<Friend>,
    pub messages_map: HashMap<String, Vec<Message>>,
}

#[cfg(feature = "ssr")]
impl Chats {
    pub async fn init(user_id: i64, store: &Store) -> Result<Self> {
        let friends = Friend::get_all(user_id, store).await?;

        let mut rooms = Vec::new();
        let mut messages_map = HashMap::new();

        let mut con = store.con.clone();

        // collect frined rooms and messages
        for friend in &friends {
            if friend.status == FriendStatus::Accepted {
                let messages = Message::list(&friend.room_id, &mut con).await?;
                let (content, send_at) = extract_latest_message(&messages);
                let room = Room {
                    key: Uuid::new_v4(),
                    id: friend.room_id.clone(),
                    name: friend.nickname.clone(),
                    cover: friend.avatar.clone(),
                    unreads: 0,
                    content,
                    send_at,
                };
                rooms.push(room);
                messages_map.insert(friend.room_id.clone(), messages);
            }
        }

        // collect user room and messages
        let user_room_id = Room::user_room_id(user_id);
        let messages = Message::list(&user_room_id, &mut con).await?;
        let (content, send_at) = extract_latest_message(&messages);
        let room = Room {
            key: Uuid::new_v4(),
            id: user_room_id.clone(),
            name: String::from("My Device"),
            cover: String::from("/default/cover.jpg"),
            unreads: 0,
            content,
            send_at,
        };
        rooms.push(room);
        messages_map.insert(user_room_id, messages);

        // sort rooms by last message
        rooms.sort_by(|a, b| a.send_at.cmp(&b.send_at));

        Ok(Chats {
            rooms,
            friends,
            messages_map,
        })
    }
}

#[cfg(feature = "ssr")]
fn extract_latest_message(messages: &Vec<Message>) -> (String, i64) {
    if let Some(last_message) = messages.last() {
        (last_message.content.clone(), last_message.send_at)
    } else {
        (String::new(), 0)
    }
}
