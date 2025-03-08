use leptos::*;
use leptos_meta::Title;
use leptos_router::Outlet;

use super::components::{Toast, Toaster};

#[component]
pub fn MainLayout() -> impl IntoView {
    provide_context(Toast::new());

    view! {
        <main id="app" class="w-full h-full p-4 min-w-fit flex flex-col items-center justify-center">
            <Outlet />
            <Toaster />
        </main>
    }
}

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <Title text="404" />

        <main class="h-full flex flex-col items-center justify-center gap-4 text-surface-on">
            <h1 class="text-8xl sm:text-9xl font-semibold text-danger" aria-label="404 Not Found">
                404
            </h1>
            <h2 class="mt-8 text-4xl sm:text-5xl font-mono font-semibold">"Ooops!"</h2>
            <h3 class="text-3xl sm:text-4xl font-semibold">"Page not found"</h3>
            <p class="my-6 text-lg sm:text-xl text-muted font-medium">"This page doesn't exist or was removed"</p>
            <a
                href="/"
                class="rounded-md py-2.5 px-6 text-sm font-medium text-primary-on bg-primary hover:bg-primary/90"
            >
                "Back to Homepage"
            </a>

        </main>
    }
}
