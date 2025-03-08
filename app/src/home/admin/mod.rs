use leptos::*;
use leptos_meta::Title;
use leptos_router::{use_location, use_navigate, Outlet, Redirect, Route};

use super::state::{DateTimeState, UserState};
use super::ADMIN_PATH;
use crate::components::icons::{HomeHouse, LockClosed, QrCode, UserGroup};
use crate::components::MenuListItem;
use crate::CHATS_PATH;
use dashboard::DashboardPage;
use qrlink::QrLinkPage;
use users::UsersPage;

mod dashboard;
mod qrlink;
mod qrlink_ops;
mod users;
mod users_ops;

const DASHBOARD_NAME: &str = "/dashboard";
const DASHBOARD_PATH: &str = "/admin/dashboard";
const USERS_NAME: &str = "/users";
const USERS_PATH: &str = "/admin/users";
const QRLINK_NAME: &str = "/qrlink";
const QRLINK_PATH: &str = "/admin/qrlink";

#[component(transparent)]
pub fn AdminRoutes() -> impl IntoView {
    view! {
        <Route path=ADMIN_PATH view=AdminLayout>
            <Route path="" view=|| view! { <Redirect path=DASHBOARD_PATH /> } />
            <Route path=DASHBOARD_NAME view=DashboardPage />
            <Route path=USERS_NAME view=UsersPage />
            <Route path=QRLINK_NAME view=QrLinkPage />
        </Route>
    }
}

#[component]
pub fn AdminLayout() -> impl IntoView {
    let user = expect_context::<UserState>().get();
    provide_context(DateTimeState::new());

    view! {
        <Title text="Admin" />

        <Show when=move || user.with(|u| u.is_admin()) fallback=|| view! { <ForbiddenPage /> }>
            <div class="grow h-full w-full flex">
                <SideBar />
                <Outlet />
            </div>
        </Show>
    }
}

#[component]
fn ForbiddenPage() -> impl IntoView {
    view! {
        <div class="grow h-full flex flex-col items-center justify-center gap-4">
            <LockClosed class="size-32 stroke-muted" />
            <h3 class="mt-8 text-4xl sm:text-5xl font-semibold">"Restricted Access"</h3>
            <p class="mb-8 text-lg sm:text-xl text-muted font-medium">"You lack permissions to access this page."</p>
            <a
                href=CHATS_PATH
                class="rounded-md py-2.5 px-6 text-sm font-medium text-primary-on bg-primary hover:bg-primary/90"
            >
                "Go to Chats"
            </a>
        </div>
    }
}

#[component]
fn SideBar() -> impl IntoView {
    let pathname = use_location().pathname;
    let navigate = use_navigate();
    let nav = Callback::new(move |url: &str| navigate(url, Default::default()));

    view! {
        <div class="shrink-0 w-64 h-full flex flex-col border-r border-border">
            <h1 class="shrink-0 p-4 font-medium text-xl">"Admin"</h1>
            <ul class="grow scrollbar scrollbar-container">
                <MenuListItem
                    active=Signal::derive(move || pathname.with(|v| v.as_str() == DASHBOARD_PATH))
                    on:click=move |_| nav.call(DASHBOARD_PATH)
                >
                    <HomeHouse class="size-5" />
                    <h3>"Overview"</h3>
                </MenuListItem>
                <MenuListItem
                    active=Signal::derive(move || pathname.with(|v| v.as_str() == USERS_PATH))
                    on:click=move |_| nav.call(USERS_PATH)
                >
                    <UserGroup class="size-5" />
                    <h3>"Users"</h3>
                </MenuListItem>
                <MenuListItem
                    active=Signal::derive(move || pathname.with(|v| v.as_str() == QRLINK_PATH))
                    on:click=move |_| nav.call(QRLINK_PATH)
                >
                    <QrCode class="size-5" />
                    <h3>"Qrlink"</h3>
                </MenuListItem>
            </ul>
        </div>
    }
}
