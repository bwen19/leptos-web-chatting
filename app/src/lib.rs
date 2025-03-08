use leptos::*;
use leptos_meta::{provide_meta_context, Body, Html, Link, Meta, Stylesheet, Title};
use leptos_router::{Route, Router, Routes};

use auth::LoginPage;
use components::provide_dark_mode;
use home::HomeRoutes;
use layout::{MainLayout, NotFoundPage};

mod auth;
mod components;
mod connection;
mod home;
mod layout;

const LOGIN_PATH: &str = "/login";
const CHATS_PATH: &str = "/chats";

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let is_dark = provide_dark_mode();

    view! {
        <Title formatter=|text| format!("{text} - Chat") />
        <Meta name="description" content="Chat: A Web App implemented by Leptos." />
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico" />
        <Stylesheet id="leptos" href="/pkg/chat.css" />

        <Html class=move || { if is_dark.get() { "dark" } else { "" } } />
        <Body class="h-screen w-full bg-container" />

        <Router fallback=|| view! { <NotFoundPage /> }>
            <Routes>
                <Route path="" view=MainLayout>
                    <HomeRoutes />
                    <Route path=LOGIN_PATH view=LoginPage />
                </Route>
            </Routes>
        </Router>
    }
}
