use leptos::*;
use leptos_meta::Title;
use leptos_router::{use_location, use_navigate, Outlet, Redirect, Route};

use super::SETTINGS_PATH;
use crate::components::icons::{LockClosed, SessionKey, UserOutline};
use crate::components::MenuListItem;
use crate::home::DateTimeState;

use profile::ProfilePage;
use security::SecurityPage;
use session::SessionPage;

mod profile;
mod security;
mod session;

const PROFILE_NAME: &str = "/profile";
const PROFILE_PATH: &str = "/settings/profile";
const SECURITY_NAME: &str = "/security";
const SECURITY_PATH: &str = "/settings/security";
const SESSION_NAME: &str = "/session";
const SESSION_PATH: &str = "/settings/session";

#[component(transparent)]
pub fn SettingsRoutes() -> impl IntoView {
    view! {
        <Route path=SETTINGS_PATH view=SettingsLayout>
            <Route path="" view=|| view! { <Redirect path=PROFILE_PATH /> } />
            <Route path=PROFILE_NAME view=ProfilePage />
            <Route path=SECURITY_NAME view=SecurityPage />
            <Route path=SESSION_NAME view=SessionPage />
        </Route>
    }
}

#[component]
pub fn SettingsLayout() -> impl IntoView {
    provide_context(DateTimeState::new());

    view! {
        <Title text="Settings" />

        <div class="grow h-full w-full flex">
            <SideBar />
            <Outlet />
        </div>
    }
}

#[component]
fn SideBar() -> impl IntoView {
    let pathname = use_location().pathname;
    let navigate = use_navigate();
    let navigator = Callback::new(move |url: &str| navigate(url, Default::default()));

    view! {
        <div class="shrink-0 w-64 h-full flex flex-col border-r border-border">
            <h1 class="shrink-0 p-4 font-medium text-xl">"Settings"</h1>
            <ul class="grow scrollbar scrollbar-container">
                <MenuListItem
                    active=Signal::derive(move || pathname.with(|v| v.as_str() == PROFILE_PATH))
                    on:click=move |_| navigator.call(PROFILE_PATH)
                >
                    <UserOutline class="size-5" />
                    <h3>"Profile"</h3>
                </MenuListItem>
                <MenuListItem
                    active=Signal::derive(move || pathname.with(|v| v.as_str() == SECURITY_PATH))
                    on:click=move |_| navigator.call(SECURITY_PATH)
                >
                    <LockClosed class="size-5" />
                    <h3>"Security"</h3>
                </MenuListItem>
                <MenuListItem
                    active=Signal::derive(move || pathname.with(|v| v.as_str() == SESSION_PATH))
                    on:click=move |_| navigator.call(SESSION_PATH)
                >
                    <SessionKey class="size-5" />
                    <h3>"Session"</h3>
                </MenuListItem>
            </ul>
        </div>
    }
}
