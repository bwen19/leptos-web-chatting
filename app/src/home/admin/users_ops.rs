use leptos::*;
use leptos_router::ActionForm;

use crate::components::icons::{DeleteTrash, EditPencil, PlusCircle, WarnTriangle};
use crate::components::{ModalWrapper, SelectLabel, Selector, Toast};
use common::{Error, FnError, InsertUserArg, UpdateUserArg, User, UserRole};

// ==================== // AddUserButton // ==================== //

#[server]
async fn add_user(arg: InsertUserArg) -> Result<(), ServerFnError<Error>> {
    use common::{ArgsValidator, AuthExtractor, StoreExtractor};

    let arg = ArgsValidator::validate(arg)?;

    let store = StoreExtractor::use_store()?;
    let _ = AuthExtractor::use_admin(false, &store).await?;

    arg.insert(&store).await?;
    Ok(())
}

#[component]
pub fn AddUserButton(refresh: RwSignal<i32>) -> impl IntoView {
    let toast = expect_context::<Toast>();
    let show_modal = create_rw_signal(false);

    let action = create_server_action::<AddUser>();
    let pending = action.pending();
    let value = action.value();

    create_effect(move |_| {
        value.with(|val| match val {
            Some(Err(FnError::WrappedServerError(e))) => toast.error(e.to_string()),
            Some(Ok(_)) => {
                toast.success(String::from("Add user successfully"));
                refresh.update(|x| *x += 1);
                show_modal.set(false)
            }
            _ => {}
        });
    });

    let role = create_rw_signal(UserRole::User);
    let role_options = store_value(vec![UserRole::User, UserRole::Admin]);

    let active = create_rw_signal(true);
    let active_options = store_value(vec![true, false]);

    view! {
        <button type="button" on:click=move |_| show_modal.set(true) class="h-8 px-4 gap-1 btn-primary">
            <PlusCircle class="size-4 stroke-2" />
            <span class="text-sm font-medium">"Add User"</span>
        </button>

        <Show when=move || show_modal.get()>
            <Portal mount=document().get_element_by_id("app").unwrap()>
                <ModalWrapper>
                    <h3 class="text-xl font-semibold tracking-tight">"Add User"</h3>
                    <p class="text-sm text-muted">"Create a new account in database"</p>

                    <ActionForm class="w-full flex flex-col gap-6 mt-6 mb-4" action>
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
                                placeholder="Enter the username"
                                class="w-full h-9 px-3 input"
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
                                placeholder="Enter the password"
                                class="w-full h-9 px-3 input"
                            />
                        </div>

                        <div class="grid grid-cols-2 gap-6 mb-3">
                            <div class="grid gap-2">
                                <label for="role" class="text-sm font-medium leading-none">
                                    Role
                                </label>
                                <input
                                    id="role"
                                    type="hidden"
                                    name="arg[role]"
                                    prop:value=move || role.with(|v| v.label())
                                />
                                <Selector value=role options=role_options />
                            </div>
                            <div class="grid gap-2">
                                <label for="active" class="text-sm font-medium leading-none">
                                    Active
                                </label>
                                <input id="active" type="hidden" name="arg[active]" prop:value=active />
                                <Selector value=active options=active_options />
                            </div>
                        </div>

                        <div class="flex items-center justify-between space-x-2">
                            <button type="button" on:click=move |_| show_modal.set(false) class="h-9 px-5 btn-ghost">
                                Cancel
                            </button>
                            <button type="submit" disabled=pending class="h-9 px-5 btn-primary">
                                "Submit"
                            </button>

                        </div>
                    </ActionForm>
                </ModalWrapper>
            </Portal>
        </Show>
    }
}

// ==================== // UpdateUserButton // ==================== //

#[server]
async fn update_user(arg: UpdateUserArg) -> Result<(), ServerFnError<Error>> {
    use common::{ArgsValidator, AuthExtractor, HubManager, StoreExtractor};

    let arg = ArgsValidator::validate(arg)?;

    let store = StoreExtractor::use_store()?;
    let _ = AuthExtractor::use_admin(false, &store).await?;

    arg.call(&store).await?;
    if let Some(active) = arg.active {
        if !active {
            HubManager::remove_user(arg.id)?;
        }
    }
    Ok(())
}

#[component]
pub fn UpdateUserButton(user: StoredValue<User>, refresh: RwSignal<i32>) -> impl IntoView {
    let toast = expect_context::<Toast>();
    let show_modal = create_rw_signal(false);

    let action = create_server_action::<UpdateUser>();
    let pending = action.pending();
    let value = action.value();

    create_effect(move |_| {
        value.with(|val| match val {
            Some(Err(FnError::WrappedServerError(e))) => toast.error(e.to_string()),
            Some(Ok(_)) => {
                toast.success(String::from("Update user successfully"));
                refresh.update(|x| *x += 1);
                show_modal.set(false)
            }
            _ => {}
        });
    });

    let role = create_rw_signal(user.with_value(|v| v.role));
    let role_options = store_value(vec![UserRole::User, UserRole::Admin]);

    let active = create_rw_signal(user.with_value(|v| v.active));
    let active_options = store_value(vec![true, false]);

    view! {
        <button type="button" on:click=move |_| show_modal.set(true) class="shrink-0 pointer-events-auto">
            <EditPencil class="size-4 hover:stroke-success" />
        </button>

        <Show when=move || show_modal.get()>
            <Portal mount=document().get_element_by_id("app").unwrap()>
                <ModalWrapper>
                    <h3 class="text-xl font-semibold tracking-tight">"Update User"</h3>
                    <p class="text-sm text-muted">"Modify the user in database"</p>

                    <ActionForm class="w-full flex flex-col gap-6 mt-6 mb-4" action>
                        <input type="hidden" name="arg[id]" value=user.with_value(|v| v.id) />
                        <div class="grid gap-2">
                            <label for="username" class="text-sm font-medium leading-none">
                                "Username: "
                                <span class="text-muted">{user.with_value(|v| v.username.clone())}</span>
                            </label>
                            <input
                                id="username"
                                type="text"
                                name="arg[username]"
                                disabled=pending
                                autocomplete="off"
                                placeholder="Change the username"
                                class="w-full h-9 px-3 input"
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
                                placeholder="Change the password"
                                class="w-full h-9 px-3 input"
                            />
                        </div>

                        <div class="grid grid-cols-2 gap-6 mb-3">
                            <div class="grid gap-2">
                                <label for="role" class="text-sm font-medium leading-none">
                                    "Role: "
                                    <span class="text-muted">{user.with_value(|v| v.role.label())}</span>
                                </label>
                                <input
                                    id="role"
                                    type="hidden"
                                    name="arg[role]"
                                    prop:value=move || role.with(|v| v.label())
                                />
                                <Selector value=role options=role_options />
                            </div>
                            <div class="grid gap-2">
                                <label for="active" class="text-sm font-medium leading-none">
                                    "Active: "
                                    <span class="text-muted">{user.with_value(|v| v.active.label())}</span>
                                </label>
                                <input id="active" type="hidden" name="arg[active]" prop:value=active />
                                <Selector value=active options=active_options />
                            </div>
                        </div>

                        <div class="flex items-center justify-between space-x-2">
                            <button type="button" on:click=move |_| show_modal.set(false) class="h-9 px-5 btn-ghost">
                                Cancel
                            </button>
                            <button type="submit" disabled=pending class="h-9 px-5 btn-primary">
                                "Submit"
                            </button>

                        </div>
                    </ActionForm>
                </ModalWrapper>
            </Portal>
        </Show>
    }
}

// ==================== // DeleteUserButton // ==================== //

#[server]
async fn delete_user(user_id: i64) -> Result<(), ServerFnError<Error>> {
    use common::{AuthExtractor, HubManager, StoreExtractor, User};

    let store = StoreExtractor::use_store()?;
    let user = AuthExtractor::use_admin(false, &store).await?;

    if user_id < 1 || user.id == user_id {
        return Err(Error::BadRequest(String::from("Invalid user id")).into());
    }
    User::delete(user_id, &store).await?;
    HubManager::remove_user(user_id)?;
    Ok(())
}

#[component]
pub fn DeleteUserButton(id: i64, refresh: RwSignal<i32>) -> impl IntoView {
    let toast = expect_context::<Toast>();
    let show_modal = create_rw_signal(false);

    let action = create_server_action::<DeleteUser>();
    let pending = action.pending();
    let value = action.value();

    create_effect(move |_| {
        value.with(|val| match val {
            Some(Err(FnError::WrappedServerError(e))) => toast.error(e.to_string()),
            Some(Ok(_)) => {
                toast.success(String::from("User has been deleted"));
                refresh.update(|x| *x += 1);
                show_modal.set(false)
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
                    <h3 class="my-1 text-center text-xl font-semibold">"Delete user"</h3>
                    <p class="mb-4 text-center text-muted">"Are you sure you want to delete this user?"</p>
                    <ActionForm class="w-full my-5" action>
                        <input type="hidden" name="user_id" value=id />
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
