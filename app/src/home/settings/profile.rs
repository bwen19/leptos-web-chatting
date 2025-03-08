use leptos::*;
use leptos_router::{ActionForm, FromFormData};
use server_fn::codec::{MultipartData, MultipartFormData};
use web_sys::{Event, FormData, SubmitEvent};

use crate::components::{Avatar, Toast};
use crate::home::UserState;
use common::{Error, FnError, UpdateUserArg, User};

#[server(input = MultipartFormData)]
pub async fn upload_avatar(data: MultipartData) -> Result<String, ServerFnError<Error>> {
    use common::{AuthExtractor, ConfigExtractor, FileManager, StoreExtractor};

    let store = StoreExtractor::use_store()?;
    let user = AuthExtractor::use_auth(false, &store).await?;

    // `.into_inner()` returns the inner `multer` stream, it is `None` if we call this on the client,
    // but always `Some(_)` on the server, so is safe to unwrap
    let mut data = data.into_inner().unwrap();

    if let Ok(Some(field)) = data.next_field().await {
        let config = ConfigExtractor::use_config()?;
        let url = FileManager::save_avatar(user.id, field.into(), config).await?;
        Ok(url)
    } else {
        Err(Error::BadRequest(String::from("No field found in multipart data")).into())
    }
}

#[server]
async fn update_profile(arg: UpdateUserArg) -> Result<User, ServerFnError<Error>> {
    use common::{ArgsValidator, AuthExtractor, ConfigExtractor, FileManager, StoreExtractor};

    let arg = ArgsValidator::validate(arg)?;

    let store = StoreExtractor::use_store()?;
    let auth_user = AuthExtractor::use_auth(false, &store).await?;

    if auth_user.id != arg.id
        || arg.username.is_some()
        || arg.password.is_some()
        || arg.role.is_some()
        || arg.active.is_some()
    {
        return Err(Error::Forbidden.into());
    }

    let rsp = arg.call(&store).await?;
    if arg.avatar.is_some() {
        let config = ConfigExtractor::use_config()?;
        FileManager::clean_avatars(rsp.id, &rsp.avatar, config).await?;
    }
    Ok(rsp)
}

#[component]
pub fn ProfilePage() -> impl IntoView {
    let user = expect_context::<UserState>().get();
    let toast = expect_context::<Toast>();

    let action = create_server_action::<UpdateProfile>();
    let pending = action.pending();
    let value = action.value();

    let avatar = create_rw_signal(user.with_untracked(|u| u.avatar.clone()));

    let candidates = (0..10)
        .map(|idx| {
            let src = store_value(format!("/default/avatar{}.png", idx));
            view! {
                <div
                    on:click=move |_| avatar.set(src.get_value())
                    class="rounded-full hover:cursor-pointer hover:ring hover:ring-success"
                >
                    <Avatar src=src.get_value() size="size-16" />
                </div>
            }
        })
        .collect_view();

    create_effect(move |_| {
        value.with(|val| match val {
            Some(Err(FnError::WrappedServerError(e))) => toast.error(e.to_string()),
            Some(Ok(ret)) => {
                user.set(ret.to_owned());
                toast.success(String::from("Profile updated"));
            }
            _ => {}
        });
    });

    let on_submit = move |ev: SubmitEvent| {
        let data = UpdateProfile::from_event(&ev).expect("failed to parse form data");
        let mut has_new = false;

        if let Some(avatar) = data.arg.avatar {
            if user.with(|u| u.avatar.ne(&avatar)) {
                has_new = true;
            }
        }
        if let Some(nickname) = data.arg.nickname {
            if user.with(|u| u.nickname.ne(&nickname)) {
                has_new = true;
            }
        }

        if !has_new {
            toast.success(String::from("Already updated"));
            ev.prevent_default();
        }
    };

    // handler for upload file
    let upload_action = create_action(|data: &FormData| {
        let data = data.clone();
        upload_avatar(data.into())
    });

    let form_node = create_node_ref::<html::Form>();
    create_effect(move |_| {
        upload_action.value().with(|val| match val {
            Some(Err(FnError::WrappedServerError(e))) => toast.error(e.to_string()),
            Some(Ok(url)) => avatar.set(url.to_owned()),
            _ => {}
        });
    });

    let on_upload = move |ev: Event| {
        ev.prevent_default();
        if let Some(form_ref) = form_node.get_untracked() {
            let form_data = FormData::new_with_form(&form_ref).unwrap();
            upload_action.dispatch(form_data);
            form_ref.reset();
        }
    };

    view! {
        <div class="px-12 py-8 grow h-full w-full">
            <h2 class="text-xl font-semibold">Profile</h2>
            <p class="my-1 text-sm text-muted">"This is how others will see you on the site."</p>

            <div class="mt-12 mb-4 text-sm font-medium leading-none">"Profile image"</div>
            <div class="flex justify-between space-x-4">
                <div class="grow flex flex-col items-center space-y-4">
                    <Avatar src=avatar size="size-32" />
                    <form node_ref=form_node>
                        <label
                            for="upload"
                            class="rounded-md h-9 px-4 py-1.5 bg-primary hover:bg-primary/90 text-primary-on text-sm font-medium cursor-pointer"
                        >
                            "Upload image"
                            <input id="upload" type="file" name="file_to_upload" on:change=on_upload class="hidden" />
                        </label>
                    </form>
                </div>
                <div>
                    <p class="text-muted text-sm font-medium mb-3">"or select an avatar:"</p>
                    <div class="grid grid-cols-5 gap-3">{candidates}</div>
                </div>
            </div>

            <ActionForm class="mt-12 grid gap-8" action on:submit=on_submit>
                <input type="hidden" name="arg[id]" prop:value=move || user.with(|u| u.id) />
                <input type="hidden" name="arg[avatar]" prop:value=avatar />
                <div class="grid gap-2">
                    <label for="username" class="text-sm font-medium leading-none">
                        Username
                    </label>
                    <input
                        id="username"
                        type="text"
                        disabled=true
                        autocomplete="off"
                        placeholder=move || user.with(|u| format!("@{}", u.username))
                        class="w-full h-11 px-3 input"
                    />
                </div>
                <div class="grid gap-2">
                    <label for="nickname" class="text-sm font-medium leading-none">
                        "Profile name: "
                        <span class="text-muted">{move || user.with(|u| u.nickname.clone())}</span>
                    </label>
                    <input
                        id="nickname"
                        type="text"
                        name="arg[nickname]"
                        disabled=pending
                        autocomplete="off"
                        placeholder="Enter your new nickname"
                        class="w-full h-11 px-3 input"
                    />
                </div>
                <div class="flex items-center justify-end">
                    <button type="submit" disabled=pending class="h-9 px-5 btn-primary">
                        "Save Changes"
                    </button>
                </div>
            </ActionForm>
        </div>
    }
}
