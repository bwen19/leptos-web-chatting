use leptos::*;

use super::DateTimeState;
use crate::components::Avatar;
use crate::home::ChatsState;
use common::Room;

#[component]
pub fn RoomEntries() -> impl IntoView {
    let chats = expect_context::<ChatsState>();
    let rooms = chats.rooms();

    let num_rooms = move || rooms.with(|c| c.len());

    view! {
        <div class="shrink-0 w-64 h-full flex flex-col border-r border-border">
            <h1 class="shrink-0 p-4 font-medium text-xl">
                "Chats " <span class="text-muted">"(" {num_rooms} ")"</span>
            </h1>
            <div class="grow scrollbar scrollbar-container">
                <ul class="flex flex-col-reverse justify-end">
                    <For
                        each=move || rooms.get()
                        key=|room| room.key
                        children=move |room| {
                            view! { <RoomItem room /> }
                        }
                    />

                </ul>
            </div>
        </div>
    }
}

#[component]
fn RoomItem(room: Room) -> impl IntoView {
    let chats = expect_context::<ChatsState>();
    let curr_room_id = chats.room_id();
    let dts = expect_context::<DateTimeState>();

    let Room {
        key: _,
        id,
        name,
        cover,
        unreads,
        content,
        send_at,
    } = room;

    let id = store_value(id);

    view! {
        <li
            on:click=move |_| curr_room_id.set(id.get_value())
            class="w-full rounded-md mb-2 px-3 py-2 flex items-center space-x-2.5 cursor-pointer hover:bg-accent"
            class=("bg-accent", move || curr_room_id.with(|v| v.as_str() == id.get_value()))
        >
            <Avatar src=cover size="size-11" />
            <div class="w-full min-w-0">
                <div class="flex items-center justify-between space-x-2">
                    <p class="truncate font-semibold">{name}</p>
                    <p class="shrink-0 text-sm text-muted">{dts.fmt_sm(send_at)}</p>
                </div>
                <div class="flex items-center justify-between space-x-2">
                    <p class="truncate text-sm text-muted">{content}</p>
                    <span
                        class="shrink-0 h-4 rounded-xl text-center bg-danger text-danger-on px-1.5 text-xs leading-tight"
                        class=("hidden", unreads == 0)
                    >
                        {unreads}
                    </span>
                </div>
            </div>
        </li>
    }
}
