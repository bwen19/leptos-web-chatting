use leptos::*;

use super::ADD_FRIEND_ID;
use crate::components::icons::ContactUsers;
use crate::components::Avatar;
use crate::home::ChatsState;
use common::{Friend, FriendStatus};

#[component]
pub fn FriendEntries() -> impl IntoView {
    let chats = expect_context::<ChatsState>();
    let curr_friend_id = chats.friend_id();

    let accepted_friends = Signal::derive(move || {
        chats.friends().with(|fds| {
            fds.iter()
                .filter(|f| f.status == FriendStatus::Accepted)
                .map(|f| f.clone())
                .collect::<Vec<Friend>>()
        })
    });
    let num_friends = move || accepted_friends.with(|x| x.len());

    view! {
        <div class="shrink-0 w-64 h-full flex flex-col border-r border-border">
            <h1 class="shrink-0 p-4 font-medium text-xl">
                "Contacts " <span class="text-muted">"(" {num_friends} ")"</span>
            </h1>
            <ul class="grow h-full scrollbar scrollbar-container">
                <li
                    on:click=move |_| curr_friend_id.set(ADD_FRIEND_ID)
                    class="w-full rounded-md mb-2 px-3 py-2 flex items-center space-x-3 cursor-pointer hover:bg-accent"
                    class=("bg-accent", move || curr_friend_id.get() == ADD_FRIEND_ID)
                >
                    <div class="shrink-0 rounded-full">
                        <div class="rounded-full flex items-center justify-center bg-success text-success-on overflow-hidden size-11">
                            <ContactUsers class="size-6 stroke-2" />
                        </div>
                    </div>
                    <div class="w-full leading-none min-w-0">
                        <p class="truncate font-semibold">"New friend"</p>
                    </div>
                </li>
                <For
                    each=move || accepted_friends.get()
                    key=|friend| friend.id
                    children=move |friend| {
                        view! { <FriendItem friend /> }
                    }
                />

            </ul>
        </div>
    }
}

#[component]
fn FriendItem(friend: Friend) -> impl IntoView {
    let curr_friend_id = expect_context::<ChatsState>().friend_id();

    let Friend {
        id,
        username,
        nickname,
        avatar,
        ..
    } = friend;

    view! {
        <li
            on:click=move |_| curr_friend_id.set(id)
            class="w-full rounded-md mb-2 px-3 py-2 flex items-center space-x-3 cursor-pointer hover:bg-accent"
            class=("bg-accent", move || curr_friend_id.get() == id)
        >
            <Avatar src=avatar size="size-11" />
            <div class="w-full min-w-0">
                <p class="truncate font-semibold">{nickname}</p>
                <p class="truncate text-sm text-muted">"@" {username}</p>
            </div>
        </li>
    }
}
