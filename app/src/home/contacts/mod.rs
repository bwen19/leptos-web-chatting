use leptos::*;
use leptos_meta::Title;

use friend::FriendPage;
use friends::FriendEntries;

mod adding;
mod friend;
mod friends;

const ADD_FRIEND_ID: i64 = -1;

#[component]
pub fn ContactsPage() -> impl IntoView {
    view! {
        <Title text="Contacts" />

        <div class="h-full w-full flex">
            <FriendEntries />
            <FriendPage />
        </div>
    }
}
