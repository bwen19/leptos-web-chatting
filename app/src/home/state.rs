use leptos::*;
use std::collections::HashMap;

use crate::connection::{provide_websocket, WebRtcState};
use common::{DateTime, Friend, Message, Room, User};

// ==================== // StateProvider // ==================== //

#[component]
pub fn StateProvider(user: User, children: Children) -> impl IntoView {
    // auth user info
    provide_context(UserState::new(user));

    // states for chats
    let chats = ChatsState::new();
    provide_context(chats);

    // state for webrtc
    let webrtc = WebRtcState::new();
    provide_context(webrtc);

    provide_websocket(chats, webrtc);

    view! { <div class="w-full h-full flex">{children()}</div> }
}

// ==================== // UserState // ==================== //

#[derive(Clone)]
pub struct UserState(RwSignal<User>);

impl Copy for UserState {}

impl UserState {
    pub fn new(user: User) -> Self {
        Self(create_rw_signal(user))
    }

    pub fn get(&self) -> RwSignal<User> {
        self.0
    }
}

// ==================== // ChatsState // ==================== //

#[derive(Clone)]
pub struct ChatsState(StoredValue<ChatsInner>);

impl Copy for ChatsState {}

#[derive(Clone)]
struct ChatsInner {
    rooms: RwSignal<Vec<Room>>,
    messages: RwSignal<HashMap<String, Vec<Message>>>,
    room_id: RwSignal<String>,
    unreads: RwSignal<u32>,
    friends: RwSignal<Vec<Friend>>,
    friend_id: RwSignal<i64>,
    adding_reqs: RwSignal<u32>,
}

impl ChatsState {
    pub fn new() -> Self {
        let inner = ChatsInner {
            rooms: create_rw_signal(Vec::new()),
            messages: create_rw_signal(HashMap::new()),
            room_id: create_rw_signal(String::new()),
            unreads: create_rw_signal(0),
            friends: create_rw_signal(Vec::new()),
            friend_id: create_rw_signal(0),
            adding_reqs: create_rw_signal(0),
        };
        Self(store_value(inner))
    }

    pub fn rooms(&self) -> RwSignal<Vec<Room>> {
        self.0.with_value(|v| v.rooms)
    }
    pub fn messages(&self) -> RwSignal<HashMap<String, Vec<Message>>> {
        self.0.with_value(|v| v.messages)
    }
    pub fn room_id(&self) -> RwSignal<String> {
        self.0.with_value(|v| v.room_id)
    }
    pub fn unreads(&self) -> RwSignal<u32> {
        self.0.with_value(|v| v.unreads)
    }
    pub fn friends(&self) -> RwSignal<Vec<Friend>> {
        self.0.with_value(|v| v.friends)
    }
    pub fn friend_id(&self) -> RwSignal<i64> {
        self.0.with_value(|v| v.friend_id)
    }
    pub fn adding_reqs(&self) -> RwSignal<u32> {
        self.0.with_value(|v| v.adding_reqs)
    }
}

// ==================== // ChatsState // ==================== //

#[derive(Clone)]
pub struct DateTimeState(StoredValue<DateTime>);

impl Copy for DateTimeState {}

impl DateTimeState {
    pub fn new() -> Self {
        Self(store_value(DateTime::now()))
    }

    pub fn fmt_sm(&self, ts: i64) -> String {
        self.0.with_value(|v| v.fmt_sm(ts))
    }

    pub fn fmt_lg(&self, ts: i64) -> String {
        self.0.with_value(|v| v.fmt_lg(ts))
    }
}
