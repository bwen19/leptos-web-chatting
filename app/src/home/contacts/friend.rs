use leptos::*;
use leptos_router::use_navigate;

use super::{adding::AddingFriend, ADD_FRIEND_ID};
use crate::components::icons::{ContactUsers, WarnTriangle};
use crate::components::{Avatar, ModalWrapper};
use crate::connection::WebSocketState;
use crate::home::ChatsState;
use crate::CHATS_PATH;
use common::{Event, Friend};

#[component]
pub fn FriendPage() -> impl IntoView {
    let chats = expect_context::<ChatsState>();
    let friends = chats.friends();
    let friend_id = chats.friend_id();

    let friend = Signal::derive(move || {
        with!(|friend_id, friends| {
            if let Some(pos) = friends.iter().position(|friend| friend.id == *friend_id) {
                friends.get(pos).cloned()
            } else {
                None
            }
        })
    });

    view! {
        {move || {
            if friend_id.get() == ADD_FRIEND_ID {
                view! { <AddingFriend /> }
            } else if let Some(friend) = friend.get() {
                view! {
                    <div class="grow w-full h-full flex flex-col">
                        <div class="shrink-0 px-6 py-4 border-b border-border font-medium text-lg text-center">
                            Friend
                        </div>
                        <FriendInfo friend />
                    </div>
                }
                    .into_view()
            } else {
                view! {
                    <div class="grow h-full w-full flex items-center justify-center">
                        <ContactUsers class="size-32 stroke-accent" />
                    </div>
                }
                    .into_view()
            }
        }}
    }
}

#[component]
fn FriendInfo(friend: Friend) -> impl IntoView {
    let ws = expect_context::<WebSocketState>();
    let chats = expect_context::<ChatsState>();

    let navigate = use_navigate();
    let show_modal = create_rw_signal(false);

    let Friend {
        id,
        username,
        nickname,
        room_id,
        avatar,
        ..
    } = friend;

    let nav_to_chat = move |_| {
        chats.room_id().set(room_id.clone());
        navigate(CHATS_PATH, Default::default());
    };

    view! {
        <div class="grow m-8 h-full rounded-md border border-border shadow-sm overflow-hidden">
            <div class="relative h-48 bg-gradient-to-r from-orange-100 to-red-100 dark:from-sky-950 dark:to-blue-950">
                <div class="absolute inset-x-0 -bottom-16 flex justify-center">
                    <div class="rounded-full ring-4 ring-surface">
                        <Avatar src=avatar size="size-32" />
                    </div>
                </div>
            </div>
            <div class="pt-20 flex flex-col items-center gap-4">
                <h3 class="text-3xl font-semibold text-surface-on">{nickname}</h3>
                <p class="text-lg font-medium text-muted">"@" {username}</p>
                <div class="mt-12 grid grad-cols-1 lg:grid-cols-2 gap-6">
                    <button type="button" on:click=move |_| show_modal.set(true) class="h-10 px-5 py-2 btn-danger">
                        "Delete Friend"
                    </button>
                    <button type="button" on:click=nav_to_chat class="h-10 px-5 py-2 btn-primary">
                        "Go Chat"
                    </button>
                </div>
            </div>
        </div>

        <Show when=move || show_modal.get()>
            <Portal mount=document().get_element_by_id("app").unwrap()>
                <ModalWrapper>
                    <div class="my-6 flex size-16 items-center justify-center rounded-full bg-danger/20">
                        <WarnTriangle class="size-10 stroke-danger" />
                    </div>
                    <h3 class="my-1 text-center text-xl font-semibold">"Delete friend"</h3>
                    <p class="mb-4 text-center text-muted">"Are you sure you want to delete this friend?"</p>
                    <button
                        type="button"
                        on:click=move |_| ws.send(Event::DeleteFriend(id))
                        class="my-5 w-full h-10 py-2 btn-primary"
                    >
                        "Confirm"
                    </button>
                    <button type="button" on:click=move |_| show_modal.set(false) class="w-full h-10 py-2 btn-outline">
                        Cancel
                    </button>
                </ModalWrapper>
            </Portal>
        </Show>
    }
}
