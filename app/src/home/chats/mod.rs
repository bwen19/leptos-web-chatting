use leptos::*;
use leptos_meta::Title;

use crate::home::DateTimeState;
use room::RoomPage;
use rooms::RoomEntries;

mod emoji;
mod messages;
mod room;
mod rooms;

#[component]
pub fn ChatsPage() -> impl IntoView {
    provide_context(DateTimeState::new());

    view! {
        <Title text="Chats" />

        <div class="grow h-full w-full flex">
            <RoomEntries />
            <RoomPage />
        </div>
    }
}
