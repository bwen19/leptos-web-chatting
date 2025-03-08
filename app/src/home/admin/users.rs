use leptos::*;

use super::users_ops::{AddUserButton, DeleteUserButton, UpdateUserButton};
use crate::components::{Avatar, BlankTable, BlankTableItem, UserActiveBadge, UserRoleBadge};
use common::{Error, ListUsersArg, ListUsersRsp, User, UserRole};

// ==================== // UsersPage // ==================== //

#[server]
async fn list_users(arg: ListUsersArg) -> Result<ListUsersRsp, ServerFnError<Error>> {
    use common::{ArgsValidator, AuthExtractor, StoreExtractor};

    let arg = ArgsValidator::validate(arg)?;

    let store = StoreExtractor::use_store()?;
    let _ = AuthExtractor::use_admin(false, &store).await?;

    let rsp = arg.call(&store).await?;
    Ok(rsp)
}

#[component]
pub fn UsersPage() -> impl IntoView {
    let refresh = create_rw_signal(0);
    let total = create_rw_signal(0);
    let arg = create_rw_signal(ListUsersArg::default());

    let users_resource = create_resource(
        move || (refresh.get(), arg.get()),
        move |(_, arg)| list_users(arg),
    );

    let num_pages = move || {
        let page_size = arg.with(|v| v.page_size);
        (total.get() + page_size - 1) / page_size
    };

    let tab_content = move || {
        users_resource
            .get()
            .map(|res: Result<ListUsersRsp, ServerFnError<Error>>| {
                match res {
                    Ok(rsp) => {
                        total.set(rsp.total);
                        let mut vlist = rsp
                            .users
                            .into_iter()
                            .map(|user| view! { <UsersTableItem user refresh /> })
                            .collect::<Vec<_>>();
                        let start = vlist.len() as i32;
                        for _ in start..arg.with_untracked(|v| v.page_size) {
                            vlist.push(view! { <BlankTableItem cols=4 /> });
                        }
                        vlist.into_view()
                    },
                    Err(e) => view! { <BlankTable rows=arg.with_untracked(|v| v.page_size) cols=4 msg=e.to_string() /> },
                }
            })
            .unwrap_or_default()
    };

    view! {
        <div class="p-6 grow h-full w-full">
            <h2 class="text-xl font-semibold">Users</h2>
            <p class="my-1 text-sm text-muted">"Manage user accounts."</p>

            <div class="mt-6 mb-4 flex items-center justify-between">
                <UserRoleSelector arg />
                <AddUserButton refresh />
            </div>
            <div class="w-full overflow-auto rounded-md border border-border">
                <table class="w-full text-sm">
                    <thead>
                        <tr class="border-b border-border text-left text-muted hover:bg-accent/50">
                            <th class="h-10 pl-7 font-medium">User</th>
                            <th class="h-10 px-2 font-medium">Role</th>
                            <th class="h-10 px-3 font-medium">Status</th>
                            <th class="h-10 px-2"></th>
                        </tr>
                    </thead>
                    <tbody>
                        <Transition fallback=move || {
                            view! { <BlankTable rows=arg.with_untracked(|v| v.page_size) cols=4 msg="Loading..." /> }
                        }>{tab_content}</Transition>
                    </tbody>
                </table>
            </div>
            <div class="mt-4 pl-1 flex items-center justify-end space-x-2 text-muted">
                <p class="grow text-sm font-medium">
                    "Page " <span>{move || arg.with(|v| v.page_id)}</span> " of " <span>{num_pages}</span>
                </p>
                <nav class="inline-flex items-center space-x-4 text-xs font-medium">
                    <button
                        type="button"
                        on:click=move |_| arg.update(|v| v.page_id -= 1)
                        disabled=move || arg.with(|v| v.page_id <= 1)
                        class="h-7 px-4 btn-outline"
                    >
                        Prev
                    </button>
                    <button
                        type="button"
                        on:click=move |_| arg.update(|v| v.page_id += 1)
                        disabled=move || arg.with(|v| v.page_id >= num_pages())
                        class="h-7 px-4 btn-outline"
                    >
                        Next
                    </button>
                </nav>
            </div>
        </div>
    }
}

// ==================== // UserRoleSelector // ==================== //

#[component]
fn UserRoleSelector(arg: RwSignal<ListUsersArg>) -> impl IntoView {
    let select = move |role: Option<UserRole>| {
        arg.update(|v| {
            v.page_id = 1;
            v.role = role;
        });
    };

    let cls = move |role: Option<UserRole>| {
        if arg.with(|v| v.role == role) {
            "rounded-md px-3 py-0.5 bg-surface text-surface-on"
        } else {
            "rounded-md px-3 py-0.5"
        }
    };

    view! {
        <div class="p-1 rounded-md bg-accent text-muted flex items-center text-sm font-medium">
            <button type="button" on:click=move |_| select(None) class=move || cls(None)>
                <span class="mx-2">"All"</span>
            </button>
            <button
                type="button"
                on:click=move |_| select(Some(UserRole::Admin))
                class=move || cls(Some(UserRole::Admin))
            >

                "Admin"
            </button>
            <button
                type="button"
                on:click=move |_| select(Some(UserRole::User))
                class=move || cls(Some(UserRole::User))
            >

                "Nomal"
            </button>
        </div>
    }
}

// ==================== // UsersTableItem // ==================== //

#[component]
fn UsersTableItem(user: User, refresh: RwSignal<i32>) -> impl IntoView {
    let user = store_value(user);

    view! {
        <tr class="last:border-b-0 border-b border-border hover:bg-accent/50">
            <td class="pl-6 h-14">
                <div class="flex items-center text-sm space-x-3">
                    <Avatar src=user.with_value(|v| v.avatar.clone()) size="size-9" />
                    <div class="leading-snug">
                        <p class="font-medium">{user.with_value(|v| v.nickname.clone())}</p>
                        <p class="text-muted">"@" {user.with_value(|v| v.username.clone())}</p>
                    </div>
                </div>
            </td>
            <td class="px-2 h-14">
                <UserRoleBadge role=user.with_value(|v| v.role) />
            </td>
            <td class="px-2 h-14">
                <UserActiveBadge active=user.with_value(|v| v.active) />
            </td>
            <td class="pr-6 h-14">
                <div class="flex items-center justify-end space-x-6 text-xs text-primary">
                    <DeleteUserButton id=user.with_value(|v| v.id) refresh />
                    <UpdateUserButton user refresh />
                </div>
            </td>
        </tr>
    }
}
