use leptos::*;
use leptos_meta::Title;

use crate::components::Logo;
use login_form::LoginForm;
use login_svg::LoginSvgImage;

mod login_form;
mod login_svg;

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <Title text="Login" />

        <div class="flex w-full max-w-md lg:max-w-4xl overflow-hidden rounded-lg bg-surface text-surface-on border border-border shadow-sm">
            <div class="p-2 basis-1/2 hidden lg:block">
                <div class="h-full rounded-md flex flex-col items-center justify-center gap-8 bg-accent">
                    <LoginSvgImage class="h-64 w-64" />
                    <h3 class="font-medium text-lg text-accent-on">"Ready to chat"</h3>
                </div>
            </div>
            <div class="grow px-16 py-14 flex flex-col justify-center">
                <div class="mb-4 flex justify-center">
                    <Logo />
                </div>
                <h3 class="font-semibold tracking-tight text-center text-2xl">"Welcome Back"</h3>
                <p class="text-sm text-center text-muted">"Please login to your account"</p>
                <LoginForm />
            </div>
        </div>
    }
}
