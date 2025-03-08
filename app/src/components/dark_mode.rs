use leptos::*;
use leptos_router::ActionForm;

use super::icons::{Moon, Sun};
use common::{CookieManager, Error, FnResult};

// ==================== // DarkMode // ==================== //

#[derive(Clone)]
pub struct DarkMode(StoredValue<DarkModeInner>);

struct DarkModeInner {
    action: Action<ToggleDarkMode, FnResult<bool>>,
    is_dark: Signal<bool>,
}

impl DarkMode {
    pub fn action(&self) -> Action<ToggleDarkMode, FnResult<bool>> {
        self.0.with_value(|v| v.action)
    }

    pub fn is_dark(&self) -> Signal<bool> {
        self.0.with_value(|v| v.is_dark)
    }
}

// ==================== // provide_dark_mode // ==================== //

#[server]
async fn toggle_dark_mode(is_dark: bool) -> Result<bool, ServerFnError<Error>> {
    CookieManager::add_darkmode(is_dark)?;
    Ok(is_dark)
}

pub fn provide_dark_mode() -> Signal<bool> {
    let initial = CookieManager::get_darkmode();

    let action = create_server_action::<ToggleDarkMode>();
    let input = action.input();
    let value = action.value();

    let darkmode_fn = move || {
        match (input.get(), value.get()) {
            // if there's some current input, use that optimistically
            (Some(submission), _) => submission.is_dark,
            // otherwise, if there was a previous value confirmed by server, use that
            (_, Some(Ok(value))) => value,
            // otherwise, use the initial value
            _ => initial,
        }
    };
    let is_dark = Signal::derive(darkmode_fn);

    provide_context(DarkMode(store_value(DarkModeInner { action, is_dark })));
    is_dark
}

// ==================== // DarkModeToggle // ==================== //

#[component]
pub fn DarkModeToggle() -> impl IntoView {
    let darkmode = expect_context::<DarkMode>();
    let action = darkmode.action();
    let is_dark = darkmode.is_dark();

    view! {
        <ActionForm action>
            <input type="hidden" name="is_dark" value=move || (!is_dark.get()).to_string() />
            <button type="submit" class="text-muted hover:text-primary">
                <Show when=move || is_dark.get() fallback=|| view! { <Moon class="size-6 " /> }>
                    <Sun class="size-6" />
                </Show>
            </button>
        </ActionForm>
    }
}
