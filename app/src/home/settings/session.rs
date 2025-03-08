use leptos::*;
use leptos_router::ActionForm;

use crate::components::icons::{DeleteTrash, SpinCircle, WarnTriangle};
use crate::components::{ModalWrapper, Toast};
use crate::home::DateTimeState;
use common::{Error, FnError, Session};

#[server]
async fn list_sessions() -> Result<Vec<Session>, ServerFnError<Error>> {
    use common::{CookieManager, StoreExtractor};

    let (user_id, session) = CookieManager::get_auth()?;

    let store = StoreExtractor::use_store()?;
    let sessions = Session::list(user_id, session, &store).await?;
    Ok(sessions)
}

#[component]
pub fn SessionPage() -> impl IntoView {
    let refresh = create_rw_signal(0);
    let rsc = create_resource(move || refresh.get(), move |_| list_sessions());

    view! {
        <div class="px-12 py-8 grow h-full w-full">
            <h2 class="text-xl font-semibold">Session</h2>
            <p class="my-1 text-sm text-muted">"Manage your sessions."</p>

            <div class="mt-12 w-full overflow-auto rounded-md border border-border">
                <Transition fallback=|| {
                    view! {
                        <div class="w-full h-56 flex items-center justify-center">
                            <SpinCircle class="animate-spin size-10" />
                        </div>
                    }
                }>
                    {move || {
                        if let Some(Ok(sessions)) = rsc.get() {
                            view! { <SessionTable sessions refresh /> }
                        } else {
                            ().into_view()
                        }
                    }}

                </Transition>
            </div>
        </div>
    }
}

#[component]
fn SessionTable(sessions: Vec<Session>, refresh: RwSignal<i32>) -> impl IntoView {
    let len = sessions.len() as i32;
    let sessions = store_value(sessions);
    let dts = expect_context::<DateTimeState>();

    view! {
        <table class="w-full text-sm">
            <thead>
                <tr class="border-b border-border text-left text-muted hover:bg-accent/50">
                    <th class="h-10 px-4 font-medium">Session</th>
                    <th class="h-10 px-2 font-medium">Status</th>
                    <th class="h-10 px-3 font-medium">"Last used"</th>
                    <th class="h-10 px-2"></th>
                </tr>
            </thead>
            <tbody>
                <For
                    each=move || sessions.get_value()
                    key=move |session| session.id.clone()
                    children=move |session| {
                        let id = store_value(session.id);
                        view! {
                            <tr class="last:border-b-0 border-b border-border hover:bg-accent/50">
                                <td class="px-4 h-12">{id.with_value(|v| v[0..8].to_owned())}</td>
                                <td class="px-2 h-12">
                                    <Show when=move || session.current>
                                        <div class="inline-flex items-center rounded-md px-2.5 py-0.5 bg-danger/90 text-danger-on text-xs font-semibold">
                                            "In use"
                                        </div>
                                    </Show>
                                </td>
                                <td class="px-2 h-12">{dts.fmt_lg(session.timestamp)}</td>
                                <td class="px-2 h-12">
                                    <div class="flex items-center justify-center">
                                        <DeleteSessionButton session=id refresh />
                                    </div>
                                </td>
                            </tr>
                        }
                    }
                />

                {(len..5)
                    .into_iter()
                    .map(|_| {
                        view! {
                            <tr class="hover:bg-accent/50">
                                <td class="px-2 h-12 w-full" colspan="4"></td>
                            </tr>
                        }
                    })
                    .collect_view()}
            </tbody>
        </table>
    }
}

// ========================================================= //

#[server]
async fn delete_session(session: String) -> Result<(), ServerFnError<Error>> {
    use common::{AuthExtractor, StoreExtractor};

    let store = StoreExtractor::use_store()?;
    let user = AuthExtractor::use_auth(false, &store).await?;

    Session::delete(user.id, session, &store).await?;
    Ok(())
}

#[component]
fn DeleteSessionButton(session: StoredValue<String>, refresh: RwSignal<i32>) -> impl IntoView {
    let toast = expect_context::<Toast>();
    let show_modal = create_rw_signal(false);

    let action = create_server_action::<DeleteSession>();
    let value = action.value();
    let pending = action.pending();

    create_effect(move |_| {
        value.with(|val| match val {
            Some(Err(FnError::WrappedServerError(e))) => toast.error(e.to_string()),
            Some(Ok(_)) => {
                toast.success(String::from("Session has been deleted"));
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
                    <h3 class="my-1 text-center text-xl font-semibold">"Delete session"</h3>
                    <p class="mb-4 text-center text-muted">"Are you sure you want to delete this session?"</p>
                    <ActionForm class="w-full my-5" action>
                        <input type="hidden" name="session" value=session.get_value() />
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
