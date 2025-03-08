use leptos::*;
use leptos_router::{ActionForm, FromFormData};
use web_sys::SubmitEvent;

use crate::components::Toast;
use crate::home::UserState;
use common::{Error, FnError, UpdatePasswordArg};

#[server]
async fn update_password(arg: UpdatePasswordArg) -> Result<(), ServerFnError<Error>> {
    use common::{ArgsValidator, AuthExtractor, StoreExtractor};

    let arg = ArgsValidator::validate(arg)?;

    let store = StoreExtractor::use_store()?;
    let auth_user = AuthExtractor::use_auth(false, &store).await?;

    if auth_user.id != arg.id {
        return Err(Error::Forbidden.into());
    }

    arg.call(&store).await?;

    Ok(())
}

#[component]
pub fn SecurityPage() -> impl IntoView {
    let user = expect_context::<UserState>().get();
    let toast = expect_context::<Toast>();

    let action = create_server_action::<UpdatePassword>();
    let pending = action.pending();
    let value = action.value();

    create_effect(move |_| {
        value.with(|val| match val {
            Some(Err(FnError::WrappedServerError(e))) => toast.error(e.to_string()),
            Some(Ok(_)) => {
                toast.success(String::from("Password updated"));
            }
            _ => {}
        });
    });

    let on_submit = move |ev: SubmitEvent| {
        let data = UpdatePassword::from_event(&ev).expect("failed to parse form data");

        if data.arg.old_password.is_empty()
            || data.arg.new_password.is_empty()
            || data.arg.confirm_password.is_empty()
        {
            toast.error(String::from("The password is empty."));
            ev.prevent_default();
        }
        if data.arg.confirm_password != data.arg.new_password {
            toast.error(String::from("The password confirmation does not match."));
            ev.prevent_default();
        }
    };

    view! {
        <div class="px-12 py-8 grow h-full w-full">
            <h2 class="text-xl font-semibold">Security</h2>
            <p class="my-1 text-sm text-muted">"Update your password below."</p>

            <ActionForm class="mt-12 grid gap-8" action on:submit=on_submit>
                <input type="hidden" name="arg[id]" prop:value=move || user.with(|u| u.id) />
                <div class="grid gap-2">
                    <label for="old-password" class="text-sm font-medium leading-none">
                        "Current password"
                    </label>
                    <input
                        id="old-password"
                        type="password"
                        name="arg[old_password]"
                        disabled=pending
                        autocomplete="off"
                        placeholder="Enter your password"
                        class="w-full h-11 px-3 input"
                    />
                </div>
                <div class="grid gap-2">
                    <label for="new-password" class="text-sm font-medium leading-none">
                        "New password"
                    </label>
                    <input
                        id="new-password"
                        type="password"
                        name="arg[new_password]"
                        disabled=pending
                        autocomplete="off"
                        placeholder="Enter your new password"
                        class="w-full h-11 px-3 input"
                    />
                </div>
                <div class="grid gap-2">
                    <label for="confirm-password" class="text-sm font-medium leading-none">
                        "Confirm password"
                    </label>
                    <input
                        id="confirm-password"
                        type="password"
                        name="arg[confirm_password]"
                        disabled=pending
                        autocomplete="off"
                        placeholder="Confirm your new password"
                        class="w-full h-11 px-3 input"
                    />
                </div>
                <div class="flex items-center justify-end">
                    <button type="submit" disabled=pending class="h-9 px-5 btn-primary">
                        "Update password"
                    </button>
                </div>
            </ActionForm>
        </div>
    }
}
