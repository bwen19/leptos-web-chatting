use leptos::*;
use leptos_router::ActionForm;
use web_sys::{Event, FormData};

use crate::components::icons::{DeleteTrash, WarnTriangle};
use crate::components::{ModalWrapper, Toast};
use common::{Error, FnError};
use server_fn::codec::{MultipartData, MultipartFormData};

// ==================== // UploadFileButton // ==================== //

#[server(input = MultipartFormData)]
async fn save_file_link(data: MultipartData) -> Result<(), ServerFnError<Error>> {
    use common::{AuthExtractor, ConfigExtractor, FileLink, HostExtractor, StoreExtractor};

    let store = StoreExtractor::use_store()?;
    let _ = AuthExtractor::use_admin(false, &store).await?;

    // `.into_inner()` returns the inner `multer` stream, it is `None` if we call this on the client,
    // but always `Some(_)` on the server, so is safe to unwrap
    let mut data = data.into_inner().unwrap();

    if let Ok(Some(field)) = data.next_field().await {
        let config = ConfigExtractor::use_config()?;
        let host = HostExtractor::use_host().await?;
        FileLink::save(field.into(), host, &store, config).await?;
        Ok(())
    } else {
        Err(Error::BadRequest(String::from("No field found in multipart data")).into())
    }
}

#[component]
pub fn UploadFileButton(refresh: RwSignal<i32>) -> impl IntoView {
    let toast = expect_context::<Toast>();

    // handler for upload file
    let upload_action: Action<FormData, Result<(), ServerFnError<Error>>> =
        create_action(|data: &FormData| {
            let data = data.clone();
            save_file_link(data.into())
        });

    let form_node = create_node_ref::<html::Form>();
    create_effect(move |_| {
        upload_action.value().with(|val| match val {
            Some(Err(FnError::WrappedServerError(e))) => toast.error(e.to_string()),
            Some(Ok(_)) => refresh.update(|x| *x += 1),
            Some(Err(e)) => toast.error(e.to_string()),
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
        <form node_ref=form_node>
            <label for="upload" class="btn-primary cursor-pointer h-7 px-4">
                "Upload file"
                <input id="upload" type="file" name="file_to_upload" on:change=on_upload class="hidden" />
            </label>
        </form>
    }
}

// ==================== // DeleteLinkButton // ==================== //

#[server]
async fn delete_link(link_id: i64) -> Result<(), ServerFnError<Error>> {
    use common::{AuthExtractor, ConfigExtractor, FileLink, StoreExtractor};

    let store = StoreExtractor::use_store()?;
    let _ = AuthExtractor::use_admin(false, &store).await?;

    let config = ConfigExtractor::use_config()?;
    FileLink::delete(link_id, &store, config).await?;
    Ok(())
}

#[component]
pub fn DeleteLinkButton(link_id: i64, refresh: RwSignal<i32>) -> impl IntoView {
    let toast = expect_context::<Toast>();
    let show_modal = create_rw_signal(false);

    let action = create_server_action::<DeleteLink>();
    let value = action.value();
    let pending = action.pending();

    create_effect(move |_| {
        value.with(|val| match val {
            Some(Err(FnError::WrappedServerError(e))) => toast.error(e.to_string()),
            Some(Ok(_)) => {
                toast.success(String::from("File and links have been deleted"));
                refresh.update(|v| *v += 1);
                show_modal.set(false);
            }
            _ => {}
        });
    });

    view! {
        <button type="button" on:click=move |_| show_modal.set(true)>
            <DeleteTrash class="size-4 hover:stroke-danger" />
        </button>

        <Show when=move || show_modal.get()>
            <Portal mount=document().get_element_by_id("app").unwrap()>
                <ModalWrapper>
                    <div class="my-6 size-16 flex items-center justify-center rounded-full bg-danger/20">
                        <WarnTriangle class="size-10 stroke-danger" />
                    </div>
                    <h3 class="my-1 text-center text-xl font-semibold">"Delete file link"</h3>
                    <p class="mb-4 text-center text-muted">"Are you sure you want to delete this file?"</p>
                    <ActionForm class="w-full my-5" action>
                        <input type="hidden" name="link_id" value=link_id />
                        <button type="submit" disabled=pending class="w-full h-9 px-3 btn-primary">
                            "Confirm"
                        </button>
                    </ActionForm>
                    <button type="button" on:click=move |_| show_modal.set(false) class="w-full h-9 px-3 btn-outline">
                        Cancel
                    </button>
                </ModalWrapper>
            </Portal>
        </Show>
    }
}
