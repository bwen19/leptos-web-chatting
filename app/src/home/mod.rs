use leptos::*;
use leptos_router::{Redirect, Route};

use crate::CHATS_PATH;
use admin::AdminRoutes;
use chats::ChatsPage;
use contacts::ContactsPage;
use layout::HomeLayout;
use settings::SettingsRoutes;

pub use state::{ChatsState, DateTimeState, UserState};

mod admin;
mod chats;
mod contacts;
mod layout;
mod navbar;
mod settings;
mod state;

const CONTACTS_PATH: &str = "/contacts";
const SETTINGS_PATH: &str = "/settings";
const ADMIN_PATH: &str = "/admin";

#[component(transparent)]
pub fn HomeRoutes() -> impl IntoView {
    view! {
        <Route path="" view=HomeLayout>
            <Route path="" view=|| view! { <Redirect path=CHATS_PATH /> } />
            <Route path=CHATS_PATH view=ChatsPage />
            <Route path=CONTACTS_PATH view=ContactsPage />
            <SettingsRoutes />
            <AdminRoutes />
        </Route>
    }
}
