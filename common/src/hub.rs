use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::{Error, Event, FeedData, FriendShip, HungUpReson, Message, Result, Room};

// ==================== // Hub // ==================== //

#[derive(Default, Clone)]
pub struct Hub(Arc<HubInner>);

#[derive(Default)]
struct HubInner {
    users: Mutex<HashMap<i64, UserState>>,
    feeds: Mutex<HashMap<String, Feed>>,
}

impl Hub {
    /// Register a new client of websocket connection
    ///
    pub fn register(
        &self,
        user_id: i64,
        client_id: Uuid,
        rooms: &Vec<Room>,
        tx: broadcast::Sender<Vec<u8>>,
    ) {
        let room_ids: HashSet<String> = rooms.iter().map(|r| r.id.clone()).collect();

        let mut users = self.0.users.lock().unwrap();
        let mut feeds = self.0.feeds.lock().unwrap();

        for room_id in &room_ids {
            match feeds.entry(room_id.clone()) {
                Entry::Occupied(mut o) => {
                    let feed = o.get_mut();
                    feed.clients.insert(client_id, tx.clone());
                }
                Entry::Vacant(v) => {
                    let mut clients = HashMap::new();
                    clients.insert(client_id, tx.clone());
                    let feed = Feed::new(clients);
                    v.insert(feed);
                }
            }
        }

        if let Some(user) = users.get_mut(&user_id) {
            user.num_clients += 1;
        } else {
            users.insert(user_id, UserState::new(room_ids));
        }
    }

    /// Unegister the client of websocket connection
    ///
    pub fn unregister(&self, user_id: i64, client_id: &Uuid) {
        let mut users = self.0.users.lock().unwrap();
        let mut feeds = self.0.feeds.lock().unwrap();

        if let Some(user) = users.get(&user_id) {
            for room_id in &user.room_ids {
                if let Some(feed) = feeds.get_mut(room_id) {
                    feed.clients.remove(client_id);
                    if feed.clients.is_empty() {
                        feeds.remove(room_id);
                    }
                }
            }
        }

        if let Some(user) = users.get_mut(&user_id) {
            user.num_clients -= 1;
            if user.num_clients <= 0 {
                users.remove(&user_id);
            }
        }
    }

    /// Remove all clients of a user
    ///
    pub fn remove(&self, user_id: i64) {
        let user_room_id = Room::user_room_id(user_id);

        let mut users = self.0.users.lock().unwrap();
        let mut feeds = self.0.feeds.lock().unwrap();

        let clients = if let Some(feed) = feeds.remove(&user_room_id) {
            feed.clients.into_keys().collect()
        } else {
            Vec::new()
        };

        if let Some(user) = users.remove(&user_id) {
            for room_id in user.room_ids {
                if let Some(feed) = feeds.get_mut(&room_id) {
                    for client_id in &clients {
                        feed.clients.remove(client_id);
                    }
                    if feed.clients.is_empty() {
                        feeds.remove(&room_id);
                    }
                }
            }
        }
    }

    /// Create a room for a friendship
    ///
    pub fn create_friend_room(&self, fsp: FriendShip) {
        let mut users = self.0.users.lock().unwrap();
        let mut feeds = self.0.feeds.lock().unwrap();

        let room_id = Room::friend_room_id(&fsp);
        if let Some(user) = users.get_mut(&fsp.id0) {
            user.room_ids.insert(room_id.clone());
        }
        if let Some(user) = users.get_mut(&fsp.id1) {
            user.room_ids.insert(room_id.clone());
        }

        // collect all online clients of the friendship
        let mut clients = HashMap::new();
        {
            let room_id = Room::user_room_id(fsp.id0);
            if let Some(feed) = feeds.get(&room_id).cloned() {
                for (client_id, tx) in feed.clients {
                    clients.insert(client_id, tx);
                }
            }

            let room_id = Room::user_room_id(fsp.id1);
            if let Some(feed) = feeds.get(&room_id).cloned() {
                for (client_id, tx) in feed.clients {
                    clients.insert(client_id, tx);
                }
            }
        }

        feeds.insert(room_id, Feed::new(clients));
    }

    /// Remove the room of a friendship
    ///
    pub fn remove_friend_room(&self, fsp: FriendShip) {
        let mut users = self.0.users.lock().unwrap();
        let mut feeds = self.0.feeds.lock().unwrap();

        let room_id = Room::friend_room_id(&fsp);
        if let Some(user) = users.get_mut(&fsp.id0) {
            user.room_ids.remove(&room_id);
        }
        if let Some(user) = users.get_mut(&fsp.id1) {
            user.room_ids.remove(&room_id);
        }

        feeds.remove(&room_id);
    }

    /// Broadcast a message in a room
    ///
    pub fn broadcast(&self, msg: Message) -> Result<Message> {
        let mut feeds = self.0.feeds.lock().unwrap();
        if let Some(feed) = feeds.get_mut(&msg.room_id) {
            let event = Event::Receive(msg.update(feed.last_send_at));

            let msg = serde_json::to_vec(&event)?;
            for sender in feed.clients.values() {
                sender.send(msg.clone())?;
            }

            if let Event::Receive(message) = event {
                feed.last_send_at = message.send_at;
                Ok(message)
            } else {
                Err(Error::InternalServer)
            }
        } else {
            Err(Error::BadRequest(String::from("The room doesn't exists!")))
        }
    }

    /// Send message to a user's all clients
    ///
    pub fn send(&self, user_id: i64, event: &Event) -> Result<()> {
        let room_id = Room::user_room_id(user_id);
        let msg = serde_json::to_vec(&event)?;

        let feeds = self.0.feeds.lock().unwrap();
        if let Some(feed) = feeds.get(&room_id) {
            for sender in feed.clients.values() {
                sender.send(msg.clone())?;
            }
        }
        Ok(())
    }

    /// Send message to a user's client
    ///
    pub fn notify(&self, user_id: i64, client_id: &Uuid, event: Event) -> Result<bool> {
        let room_id = Room::user_room_id(user_id);
        let msg = serde_json::to_vec(&event)?;

        let feeds = self.0.feeds.lock().unwrap();
        if let Some(feed) = feeds.get(&room_id) {
            if let Some(sender) = feed.clients.get(&client_id) {
                sender.send(msg)?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Change callable to false of the two users
    ///
    pub fn make_call(&self, caller_id: i64, called_id: i64) -> Result<HungUpReson> {
        let mut users = self.0.users.lock().unwrap();

        if let Some(caller) = users.get(&caller_id) {
            if !caller.callable {
                return Ok(HungUpReson::Busy);
            }
        } else {
            return Err(Error::InternalServer);
        }

        if let Some(called) = users.get_mut(&called_id) {
            if !called.callable {
                return Ok(HungUpReson::Busy);
            }
            called.callable = false;
        } else {
            return Ok(HungUpReson::Offline);
        };

        if let Some(caller) = users.get_mut(&caller_id) {
            caller.callable = false;
        }

        Ok(HungUpReson::Finish)
    }

    /// Change callable to true of the two users
    ///
    pub fn make_hung_up(&self, caller_id: i64, called_id: i64) -> Result<()> {
        let mut users = self.0.users.lock().unwrap();

        if let Some(caller) = users.get_mut(&caller_id) {
            caller.callable = true;
        }

        if let Some(called) = users.get_mut(&called_id) {
            called.callable = true;
        }

        Ok(())
    }

    /// Get first n feeds in the Hub
    ///
    pub fn get_feeds(&self, num: usize) -> (i32, Vec<FeedData>) {
        let feeds = self.0.feeds.lock().unwrap();
        let num_feeds = feeds.len() as i32;

        let data: Vec<FeedData> = feeds
            .iter()
            .take(num)
            .map(|(name, feed)| FeedData {
                name: name.to_owned(),
                num_clients: feed.clients.len() as i32,
                active_at: feed.last_send_at,
            })
            .collect();

        (num_feeds, data)
    }

    /// Get first n friends in the Hub
    ///
    pub fn get_users(&self, num: usize) -> (i32, i32, Vec<i64>) {
        let users = self.0.users.lock().unwrap();

        let mut num_clients = 0;
        let mut count = 0;
        let mut user_ids = Vec::with_capacity(num);

        for (user_id, user) in users.iter() {
            if count < num {
                user_ids.push(*user_id);
            }
            count += 1;
            num_clients += user.num_clients;
        }

        (num_clients, count as i32, user_ids)
    }
}

// ==================== // UserState // ==================== //

#[derive(Clone)]
struct UserState {
    callable: bool,
    num_clients: i32,
    room_ids: HashSet<String>,
}

impl UserState {
    fn new(room_ids: HashSet<String>) -> Self {
        Self {
            callable: true,
            num_clients: 1,
            room_ids,
        }
    }
}

// ==================== // Feed // ==================== //

#[derive(Clone)]
struct Feed {
    last_send_at: i64,
    clients: HashMap<Uuid, broadcast::Sender<Vec<u8>>>,
}

impl Feed {
    fn new(clients: HashMap<Uuid, broadcast::Sender<Vec<u8>>>) -> Self {
        Self {
            last_send_at: 0,
            clients,
        }
    }
}
