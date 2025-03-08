use leptos::{ev::SubmitEvent, *};
use leptos_router::{ActionForm, FromFormData};

use crate::components::icons::{CloseXmark, SearchGlass};
use crate::components::{Avatar, Toast};
use crate::connection::WebSocketState;
use crate::home::{ChatsState, UserState};
use common::{Error, Event, Friend, FriendStatus, User};

// ========================================================= //

#[component]
pub fn AddingFriend() -> impl IntoView {
    view! {
        <div class="w-full h-full flex flex-col">
            <div class="shrink-0 px-6 py-4 border-b border-border font-medium text-lg text-center">"New friend"</div>
            <div class="grow scrollbar scrollbar-container">
                <SearchFriendSection />
                <div class="relative flex p-3 items-center">
                    <div class="flex-grow border-t border-border"></div>
                    <span class="flex-shrink mx-4 text-muted text-sm">Pending</span>
                    <div class="flex-grow border-t border-border"></div>
                </div>
                <PendingFriends />
            </div>
        </div>
    }
}

// ========================================================= //

#[server]
async fn search_user(key: String) -> Result<Option<User>, ServerFnError<Error>> {
    use common::{StoreExtractor, User};

    let store = StoreExtractor::use_store()?;
    let user = User::find(&key, &store).await?.map(|user| user.into());
    Ok(user)
}

#[component]
fn SearchFriendSection() -> impl IntoView {
    let toast = expect_context::<Toast>();
    let ws = expect_context::<WebSocketState>();

    let auth_user = expect_context::<UserState>().get();
    let chats = expect_context::<ChatsState>();
    let friends = chats.friends();

    let action = create_server_action::<SearchUser>();
    let pending = action.pending();
    let value = action.value();

    let info = move || {
        with!(|value, auth_user, friends| {
            if let Some(Ok(Some(user))) = value {
                if user.id == auth_user.id {
                    Some("You")
                } else if let Some(_) = friends.iter().position(|friend| friend.id == user.id) {
                    Some("Already in the list")
                } else {
                    None
                }
            } else {
                Some("Nothing found")
            }
        })
    };

    let on_submit = move |ev: SubmitEvent| {
        let data = SearchUser::from_event(&ev).expect("failed to parse form data");
        if data.key.is_empty() {
            toast.error(String::from("Please input a name to search"));
            ev.prevent_default();
        }
    };

    view! {
        <ActionForm class="p-4 flex gap-4" action on:submit=on_submit>
            <div class="grow relative">
                <div class="pointer-events-none absolute inset-y-0 left-0 w-10 flex items-center justify-center">
                    <SearchGlass class="size-4 text-muted" />
                </div>
                <input
                    type="text"
                    name="key"
                    disabled=pending
                    autocomplete="off"
                    spellcheck=false
                    placeholder="Search user..."
                    class="w-full px-10 h-10 input"
                />
                <button type="reset" class="absolute inset-y-0 right-0 flex w-10 items-center justify-center">
                    <CloseXmark class="size-4 stroke-muted hover:stroke-primary" />
                </button>
            </div>
            <button type="submit" class="shrink-0 h-10 px-4 btn-primary" disabled=pending>
                "Search"
            </button>
        </ActionForm>

        <div class="px-4 mb-2">
            {move || {
                if let Some(Ok(Some(user))) = value.get() {
                    view! {
                        <div class="flex w-full rounded-md items-center space-x-4 py-3 px-4 bg-accent">
                            <div class="shrink-0 rounded-full">
                                <Avatar src=user.avatar />
                            </div>
                            <div class="w-36">
                                <p class="truncate font-semibold">{user.nickname}</p>
                                <p class="truncate text-sm text-muted">{user.username}</p>
                            </div>
                            <div class="grow text-sm text-muted">{info}</div>
                            <button
                                type="button"
                                on:click=move |_| ws.send(Event::AddFriend(user.id))
                                disabled=move || info().is_some()
                                class="shrink-0 h-9 px-5 btn-primary"
                            >
                                "Add friend"
                            </button>
                        </div>
                    }
                        .into_view()
                } else {
                    view! {
                        <p class="text-center my-2 text-sm text-muted font-medium">"No user found for your search"</p>
                    }
                        .into_view()
                }
            }}

        </div>
    }
}

// ========================================================= //

#[component]
fn PendingFriends() -> impl IntoView {
    let chats = expect_context::<ChatsState>();

    let pending_friends = move || {
        chats.friends().with(|fds| {
            fds.iter()
                .filter(|f| f.status != FriendStatus::Accepted)
                .map(|f| f.clone())
                .collect::<Vec<Friend>>()
        })
    };

    view! {
        <ul class="px-4 py-2">
            <For
                each=pending_friends
                key=|friend| friend.id
                children=move |friend| {
                    view! { <FriendItem friend /> }
                }
            />

        </ul>
    }
}

#[component]
fn FriendItem(friend: Friend) -> impl IntoView {
    let ws = expect_context::<WebSocketState>();

    let Friend {
        id,
        username,
        nickname,
        avatar,
        status,
        room_id: _,
    } = friend;

    let info = move || {
        if status == FriendStatus::Added {
            "Waiting for a response..."
        } else {
            "Waiting to be processed"
        }
    };

    let hint = move || {
        if status == FriendStatus::Added {
            "Revert"
        } else {
            "Refuse"
        }
    };

    view! {
        <li class="last:mb-0 mb-4 flex w-full rounded-md items-center space-x-4 py-3 px-4 bg-accent">
            <div class="shrink-0 rounded-full">
                <Avatar src=avatar />
            </div>
            <div class="w-36">
                <p class="truncate font-semibold">{nickname}</p>
                <p class="truncate text-sm text-muted">{username}</p>
            </div>
            <div class="grow text-sm text-muted">{info}</div>
            <button
                type="button"
                on:click=move |_| ws.send(Event::RevertFriend(id))
                class="shrink-0 h-7 px-5 py-1 btn-danger"
            >
                {hint}
            </button>
            <button
                type="button"
                on:click=move |_| ws.send(Event::AcceptFriend(id))
                disabled=status == FriendStatus::Added
                class="shrink-0 h-7 px-5 py-1 btn-primary"
            >
                Accept
            </button>
        </li>
    }
}
