use leptos::*;
use leptos_router::Outlet;
use server_fn::codec::GetUrl;

use super::navbar::Navbar;
use super::state::StateProvider;
use crate::components::icons::SpinCircle;
use crate::components::Logo;
use crate::connection::CallSection;
use crate::LOGIN_PATH;
use common::{Error, User};

#[server(input = GetUrl)]
async fn get_auth_user() -> Result<User, ServerFnError<Error>> {
    use common::{AuthExtractor, StoreExtractor};

    let store = StoreExtractor::use_store()?;
    AuthExtractor::use_auth(true, &store).await
}

#[component]
pub fn HomeLayout() -> impl IntoView {
    let user_res = create_resource(|| (), move |_| get_auth_user());

    view! {
        <div class="w-full h-full max-w-[1080px] max-h-[780px] overflow-hidden rounded-lg bg-surface text-surface-on border border-border shadow-sm">
            <Suspense fallback=|| {
                view! {
                    <div class="w-full h-full flex items-center justify-center">
                        <SpinCircle class="animate-spin size-12" />
                    </div>
                }
            }>
                {move || {
                    if let Some(Ok(user)) = user_res.get() {
                        view! {
                            <StateProvider user>
                                <CallSection />
                                <Navbar />
                                <Outlet />
                            </StateProvider>
                        }
                    } else {
                        view! { <WelcomePage /> }
                    }
                }}

            </Suspense>
        </div>
    }
}

#[component]
fn WelcomePage() -> impl IntoView {
    view! {
        <div class="w-full h-full flex flex-col items-center justify-center gap-4">
            <Logo size="size-16" />
            <h3 class="mt-6 text-4xl font-semibold">"Welcome to Chat"</h3>
            <p class="mb-8 text-2xl text-muted font-medium">"Liberate your communication"</p>
            <a
                href=LOGIN_PATH
                class="rounded-md bg-primary py-2.5 px-6 text-sm text-primary-on font-medium hover:bg-primary/90"
            >
                "Get started"
            </a>
        </div>
    }
}
