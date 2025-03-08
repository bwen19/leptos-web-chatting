use leptos::*;
use leptos_router::{ActionForm, FromFormData};

use crate::components::Toast;
use common::{Error, FnError, LoginArg};

// ==================== // LoginForm // ==================== //

#[server]
async fn login(arg: LoginArg) -> Result<(), ServerFnError<Error>> {
    use crate::CHATS_PATH;
    use common::{ArgsValidator, CookieManager, StoreExtractor};

    let arg = ArgsValidator::validate(arg)?;
    let store = StoreExtractor::use_store()?;

    let (session, user) = arg.call(&store).await?;
    CookieManager::add_auth(user.id, session)?;

    leptos_axum::redirect(CHATS_PATH);
    Ok(())
}

#[component]
pub fn LoginForm() -> impl IntoView {
    let toast = expect_context::<Toast>();

    let action = create_server_action::<Login>();
    let pending = action.pending();
    let value = action.value();

    create_effect(move |_| {
        value.with(|val| {
            if let Some(Err(FnError::WrappedServerError(e))) = val {
                toast.error(e.to_string());
            }
        });
    });

    let on_submit = move |ev: ev::SubmitEvent| {
        let data = Login::from_event(&ev).expect("failed to parse form data");
        if data.arg.username.is_empty() {
            toast.info(String::from("Username is blank"));
            ev.prevent_default();
        } else if data.arg.password.is_empty() {
            toast.info(String::from("Password is blank"));
            ev.prevent_default();
        }
    };

    view! {
        <ActionForm class="mt-6 grid gap-6" action on:submit=on_submit>
            <div class="grid gap-2">
                <label for="username" class="text-sm font-medium leading-none">
                    Username
                </label>
                <input
                    id="username"
                    type="text"
                    name="arg[username]"
                    disabled=pending
                    autocomplete="off"
                    placeholder="Enter your username"
                    class="w-full h-11 px-3 input"
                />
            </div>

            <div class="grid gap-2">
                <label for="password" class="text-sm font-medium leading-none">
                    Password
                </label>
                <input
                    id="password"
                    type="password"
                    name="arg[password]"
                    disabled=pending
                    autocomplete="off"
                    placeholder="Enter your password"
                    class="w-full h-11 px-3 input"
                />
            </div>

            <button type="submit" disabled=pending class="mt-4 w-full h-11 btn-primary">
                "Sign in"
            </button>
        </ActionForm>
    }
}
