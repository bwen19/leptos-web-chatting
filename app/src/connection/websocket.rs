use leptos::*;
use web_sys::WebSocket;

use super::WebRtcState;
use crate::home::ChatsState;
use common::Event;

// ==================== // WebSocketState // ==================== //

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SocketStatus {
    Open,
    Closed,
    Idle,
}

#[derive(Clone)]
pub struct WebSocketState(StoredValue<WebSocketInner>);

#[derive(Clone)]
struct WebSocketInner {
    ws: Option<WebSocket>,
    status: RwSignal<SocketStatus>,
}

impl Copy for WebSocketState {}

impl WebSocketState {
    pub fn new() -> Self {
        let inner = WebSocketInner {
            ws: None,
            status: create_rw_signal(SocketStatus::Idle),
        };
        Self(store_value(inner))
    }

    /// Returns the status Signal of WebSocket
    ///
    pub fn status(&self) -> RwSignal<SocketStatus> {
        self.0.with_value(|v| v.status)
    }

    /// Send WebSocket Event to server
    ///
    pub fn send(&self, evt: Event) {
        let inner = self.0.get_value();
        if inner.status.get_untracked() == SocketStatus::Open {
            if let Some(ws) = inner.ws {
                if let Ok(msg) = serde_json::to_vec(&evt) {
                    let _ = ws.send_with_u8_array(&msg);
                }
            }
        }
    }

    /// Reconnect WebSocket Server
    ///
    pub fn reconnect(&self) {
        let status = self.0.with_value(|v| v.status);
        if status.get_untracked() == SocketStatus::Closed {
            status.set(SocketStatus::Idle);
        }
    }
}

// ==================== // provide_websocket // ==================== //

pub fn provide_websocket(chats: ChatsState, webrtc: WebRtcState) {
    #[cfg(feature = "ssr")]
    {
        let _ = chats;
        let _ = webrtc;
        provide_websocket_0();
    }

    #[cfg(feature = "hydrate")]
    provide_websocket_1(chats, webrtc);
}

#[cfg(feature = "ssr")]
fn provide_websocket_0() {
    provide_context(WebSocketState::new());
}

#[cfg(feature = "hydrate")]
fn provide_websocket_1(chats: ChatsState, webrtc: WebRtcState) {
    use std::rc::Rc;

    use leptos_router::use_location;
    use wasm_bindgen::{prelude::*, UnwrapThrowExt};
    use web_sys::{js_sys, BinaryType, Event as WsEvent, MessageEvent};

    use super::RtcStatus;
    use crate::components::Toast;
    use crate::CHATS_PATH;
    use common::{FriendStatus, HungUpReson};

    let ws_state = WebSocketState::new();
    let status = ws_state.status();

    let toast = expect_context::<Toast>();
    let pathname = use_location().pathname;

    // get ws connect url
    let location = window().location();
    let protocol = location
        .protocol()
        .expect("Protocol not found")
        .replace("http", "ws");
    let host = location.host().expect("Host not found");
    let url = format!("{}//{}/ws", protocol, host);

    // create connect function
    let connect_ref: StoredValue<Option<Rc<dyn Fn()>>> = store_value(None);

    connect_ref.set_value(Some(Rc::new(move || {
        let ws = WebSocket::new(&url).unwrap_throw();
        ws.set_binary_type(BinaryType::Arraybuffer);

        // onopen handler
        let onopen_callback = Closure::<dyn FnMut(_)>::new(move |_: WsEvent| {
            toast.success(String::from("WebSocket connected"));
            status.set(SocketStatus::Open);
        });
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        // onmessage handler
        let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() else {
                return;
            };
            let array = js_sys::Uint8Array::new(&abuf);

            let Ok(event) = serde_json::from_slice::<Event>(&array.to_vec()) else {
                return;
            };
            match event {
                Event::InitRooms(rooms) => chats.rooms().set(rooms),
                Event::InitFriends(friends) => {
                    let num = friends
                        .iter()
                        .filter(|x| x.status == FriendStatus::Adding)
                        .count() as u32;
                    chats.friends().set(friends);
                    chats.adding_reqs().set(num);
                }
                Event::InitMessages(messages) => chats.messages().set(messages),
                Event::Receive(message) => {
                    let incr = !(pathname.get_untracked() == CHATS_PATH
                        && chats.room_id().get_untracked() == message.room_id);
                    if incr {
                        chats.unreads().update(|v| *v += 1);
                    }

                    chats.rooms().update(|rooms| {
                        let Some(pos) = rooms.iter().position(|v| v.id == message.room_id) else {
                            return;
                        };

                        let room = rooms.remove(pos).update(&message, incr);
                        rooms.push(room);
                    });

                    chats.messages().update(|messages_map| {
                        if let Some(messages) = messages_map.get_mut(&message.room_id) {
                            messages.push(message);
                        }
                    })
                }
                Event::ReceiveRoom(room) => {
                    chats.friends().update(|friends| {
                        if let Some(friend) = friends.iter_mut().find(|v| v.room_id == room.id) {
                            if friend.status == FriendStatus::Adding {
                                chats.adding_reqs().update(|v| *v -= 1);
                            }
                            friend.status = FriendStatus::Accepted;
                        }
                    });
                    chats.messages().update(|messages| {
                        messages.insert(room.id.clone(), Vec::new());
                    });
                    chats.rooms().update(|rooms| rooms.push(room));
                }
                Event::ReceiveFriend(friend) => {
                    if friend.status == FriendStatus::Adding {
                        chats.adding_reqs().update(|v| *v += 1);
                    }
                    chats.friends().update(|friends| friends.push(friend));
                }
                Event::RevertFriend(friend_id) => {
                    chats.friends().update(|friends| {
                        let Some(pos) = friends.iter().position(|v| v.id == friend_id) else {
                            return;
                        };

                        let friend = friends.remove(pos);
                        if friend.status == FriendStatus::Adding {
                            chats.adding_reqs().update(|v| *v -= 1);
                        }
                    });
                }
                Event::DeleteFriend(friend_id) => {
                    chats.friends().update(|friends| {
                        let Some(pos) = friends.iter().position(|v| v.id == friend_id) else {
                            return;
                        };

                        let friend = friends.remove(pos);
                        chats.rooms().update(|rooms| {
                            let Some(idx) = rooms.iter().position(|v| v.id == friend.room_id)
                            else {
                                return;
                            };

                            let room = rooms.remove(idx);
                            if room.unreads > 0 {
                                chats.unreads().update(|unreads| *unreads -= room.unreads);
                            }
                        });

                        chats.room_id().update(|room_id| {
                            if room_id.as_str() == friend.room_id {
                                *room_id = String::new();
                            }
                        });
                        toast.success(format!("'{}' has been removed", friend.nickname));
                    });
                    chats.friend_id().update(|v| {
                        if *v == friend_id {
                            *v = 0;
                        }
                    });
                }
                Event::ReceiveCall(user_id, client_id) => webrtc.receive_call(user_id, client_id),
                Event::SendCallDone(user_id) => webrtc.send_call_done(user_id),
                Event::ReceiveHungUp(reson) => {
                    let rtc_status = webrtc.status();
                    match reson {
                        HungUpReson::Offline => toast.error(String::from("Your friend is offline")),
                        HungUpReson::Busy => toast.error(String::from("Your friend is busy now")),
                        HungUpReson::Refuse => {
                            if rtc_status.get_untracked() == RtcStatus::Caller {
                                toast.info(String::from("The call was declined"));
                            }
                        }
                        HungUpReson::Cancel => {
                            if rtc_status.get_untracked() == RtcStatus::Callee {
                                toast.info(String::from("The call was canceled"));
                            }
                        }
                        HungUpReson::Finish => {
                            toast.info(String::from("The call is finished"));
                        }
                    }
                    webrtc.receive_hung_up();
                }
                Event::ReceiveReply(client_id) => webrtc.send_offer(client_id, ws_state),
                Event::ReceiveOffer(offser) => webrtc.send_answer(offser, ws_state),
                Event::ReceiveAnswer(answer) => webrtc.receive_answer(answer),
                Event::ReceiveCandidate(candidate) => webrtc.receive_candidate(candidate),
                _ => {}
            }
        });
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        // onerror handler
        let onerror_callback = Closure::<dyn FnMut(_)>::new(move |_: WsEvent| {
            status.set(SocketStatus::Closed);
            toast.error(String::from("WebSocket Disconnected"));
        });
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        // onclose handler
        let onclose_callback = Closure::<dyn FnMut(_)>::new(move |_: WsEvent| {
            status.set(SocketStatus::Closed);
        });
        ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();

        ws_state.0.update_value(|v| v.ws = Some(ws));
    })));

    create_effect(move |_| {
        if status.get() == SocketStatus::Idle {
            if let Some(connect) = connect_ref.get_value() {
                connect();
            }
        }
    });
    on_cleanup(move || {
        ws_state.0.update_value(|v| {
            v.ws.as_ref().inspect(|x| {
                let _ = x.close();
            });
            v.ws = None;
        });
        status.set(SocketStatus::Closed);
    });

    provide_context(ws_state);
}
