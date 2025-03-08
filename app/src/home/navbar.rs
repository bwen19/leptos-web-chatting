use leptos::*;
use leptos_dom::helpers::TimeoutHandle;
use leptos_router::{use_location, use_navigate, ActionForm};
use std::time::Duration;

use super::state::{ChatsState, UserState};
use super::{ADMIN_PATH, CONTACTS_PATH, SETTINGS_PATH};
use crate::components::icons::{AdminSquares, ChatBubble, ContactUsers, SettingsCog, SignOut};
use crate::components::{Avatar, DarkModeToggle, Logo, ModalWrapper, UserRoleBadge};
use crate::connection::{SocketStatus, WebSocketState};
use crate::CHATS_PATH;
use common::Error;

// ==================== // Navbar // ==================== //

#[component]
pub fn Navbar() -> impl IntoView {
    let pathname = use_location().pathname;
    let navigate = use_navigate();
    let navigator = Callback::new(move |url: &str| navigate(url, Default::default()));

    let user = expect_context::<UserState>().get();
    let chats = expect_context::<ChatsState>();

    view! {
        <div class="shrink-0 w-16 flex flex-col items-center py-4 border-r border-border">
            <div class="shrink-0 h-28">
                <Logo />
            </div>
            <nav class="grow flex flex-col items-center justify-start gap-4">
                <NavIconButton
                    value=chats.unreads()
                    active=Signal::derive(move || pathname.with(|v| v.as_str() == CHATS_PATH))
                    on:click=move |_| navigator.call(CHATS_PATH)
                >
                    <ChatBubble class="size-6" />
                </NavIconButton>
                <NavIconButton
                    value=chats.adding_reqs()
                    active=Signal::derive(move || pathname.with(|v| v.as_str() == CONTACTS_PATH))
                    on:click=move |_| navigator.call(CONTACTS_PATH)
                >
                    <ContactUsers class="size-6" />
                </NavIconButton>
                <NavIconButton
                    active=Signal::derive(move || pathname.with(|v| v.as_str().starts_with(SETTINGS_PATH)))
                    on:click=move |_| navigator.call(SETTINGS_PATH)
                >
                    <SettingsCog class="size-6" />
                </NavIconButton>
                <NavIconButton
                    hidden=Signal::derive(move || user.with(|v| !v.is_admin()))
                    active=Signal::derive(move || pathname.with(|v| v.as_str().starts_with(ADMIN_PATH)))
                    on:click=move |_| navigator.call(ADMIN_PATH)
                >
                    <AdminSquares class="size-6" />
                </NavIconButton>
            </nav>
            <div class="shrink-0 flex flex-col items-center justify-between gap-6">
                <DarkModeToggle />
                <LogoutButton />
                <AuthUserStatus />
            </div>
        </div>
    }
}

// ==================== // NavIconButton // ==================== //

#[component]
fn NavIconButton(
    #[prop(default = false.into(), into)] active: MaybeSignal<bool>,
    #[prop(default = 0.into(), into)] value: MaybeSignal<u32>,
    #[prop(default = false.into(), into)] hidden: MaybeSignal<bool>,
    children: Children,
) -> impl IntoView {
    let btn = move || {
        if hidden.get() {
            "hidden"
        } else if active.get() {
            "relative rounded-lg p-2 text-accent-on bg-accent"
        } else {
            "relative rounded-lg p-2 text-muted hover:text-primary"
        }
    };

    let bdg = move || {
        if value.get() > 0 {
            "absolute z-10 top-1 right-1 rounded-full size-2 bg-danger"
        } else {
            ""
        }
    };

    view! {
        <button class=btn>
            <span class=bdg></span>
            {children()}
        </button>
    }
}

// ==================== // LogoutButton // ==================== //

#[server]
async fn logout() -> Result<(), ServerFnError<Error>> {
    use crate::LOGIN_PATH;
    use common::{CookieManager, Session, StoreExtractor};

    let (user_id, session) = CookieManager::remove_auth()?;
    if user_id > 0 && !session.is_empty() {
        let store = StoreExtractor::use_store()?;
        Session::delete(user_id, session, &store).await?;
    }
    leptos_axum::redirect(LOGIN_PATH);
    Ok(())
}

#[component]
fn LogoutButton() -> impl IntoView {
    let action = create_server_action::<Logout>();
    let pending = action.pending();

    let show_modal = create_rw_signal(false);

    view! {
        <button type="button" on:click=move |_| show_modal.set(true)>
            <SignOut class="size-6 stroke-muted hover:stroke-primary" />
        </button>

        <Show when=move || show_modal.get()>
            <Portal mount=document().get_element_by_id("app").unwrap()>
                <ModalWrapper>
                    <div class="my-6 size-16 flex items-center justify-center rounded-full bg-danger/20">
                        <SignOut class="size-10 stroke-danger" />
                    </div>
                    <h3 class="my-1 text-center text-xl font-semibold">"Leaving so soon?"</h3>
                    <p class="mb-4 text-center text-muted">"Are you sure you want to log out?"</p>
                    <ActionForm class="w-full my-5" action>
                        <button type="submit" disabled=pending class="w-full h-9 btn-primary">
                            "Log out"
                        </button>
                    </ActionForm>
                    <button
                        type="button"
                        disabled=pending
                        on:click=move |_| show_modal.set(false)
                        class="w-full h-9 btn-outline"
                    >
                        Cancel
                    </button>

                </ModalWrapper>
            </Portal>
        </Show>
    }
}

// ==================== // AuthUserStatus // ==================== //

#[component]
fn AuthUserStatus() -> impl IntoView {
    let user = expect_context::<UserState>().get();
    let src = create_read_slice(user, |user| user.avatar.clone());

    let ws = expect_context::<WebSocketState>();
    let status = ws.status();
    let btn_ring = move || {
        if status.get() == SocketStatus::Open {
            "relative rounded-full ring ring-success"
        } else {
            "relative rounded-full ring ring-danger"
        }
    };

    // handle display popover
    let show_info = create_rw_signal(false);
    let handler = store_value(None::<TimeoutHandle>);

    let on_mouse_enter = move |_| {
        if show_info.get_untracked() {
            handler.update_value(|v| {
                if let Some(ref h) = v {
                    h.clear();
                    *v = None;
                }
            });
        } else {
            set_timeout(move || show_info.set(true), Duration::from_millis(200));
        }
    };
    let on_mouse_leave = move |_| {
        let h =
            set_timeout_with_handle(move || show_info.set(false), Duration::from_millis(160)).ok();
        handler.update_value(|v| *v = h);
    };

    view! {
        <button
            on:mouseenter=on_mouse_enter
            on:mouseleave=on_mouse_leave
            class=btn_ring
            disabled=move || status.get() != SocketStatus::Closed
            on:click=move |_| ws.reconnect()
        >
            <Avatar src />
            <AnimatedShow
                when=show_info
                show_class="animate-fade-in"
                hide_class="animate-fade-out"
                hide_delay=Duration::from_millis(150)
            >
                <div class="absolute z-30 overflow-hidden bottom-2 left-12 rounded-md p-4 bg-primary">
                    <div class="mb-4 flex items-center space-x-2">
                        <Avatar src />
                        <div class="grow text-sm text-left">
                            <p class="text-primary-on font-semibold">{user.with(|v| v.nickname.clone())}</p>
                            <p class="text-muted font-medium">"@" {user.with(|v| v.username.clone())}</p>
                        </div>
                        <UserRoleBadge role=user.with(|v| v.role) />
                    </div>

                    <Show
                        when=move || status.get() == SocketStatus::Open
                        fallback=|| {
                            view! {
                                <div class="w-fit flex items-center space-x-2">
                                    <div class="size-3 rounded-full bg-danger"></div>
                                    <p class="text-sm text-muted font-medium text-left text-nowrap">
                                        "Connection Closed"
                                    </p>
                                </div>
                            }
                        }
                    >

                        <div class="w-fit flex items-center space-x-2">
                            <div class="size-3 rounded-full bg-success"></div>
                            <p class="text-sm text-muted font-medium text-left text-nowrap">"WebSocket Connected"</p>
                        </div>
                    </Show>
                </div>
            </AnimatedShow>
        </button>
    }
}
