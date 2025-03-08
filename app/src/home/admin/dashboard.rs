use leptos::*;
use leptos_router::ActionForm;

use crate::components::icons::{DeleteTrash, RefreshArrow, SpinCircle};
use crate::components::{Avatar, Toast, UserRoleBadge};
use crate::home::DateTimeState;
use common::{Error, FeedData, FnError, HubData, User};

#[server]
async fn get_hub_data() -> Result<HubData, ServerFnError<Error>> {
    use common::{AuthExtractor, HubManager, StoreExtractor};

    let store = StoreExtractor::use_store()?;
    let _ = AuthExtractor::use_admin(false, &store).await?;

    HubManager::get_data(&store).await
}

#[component]
pub fn DashboardPage() -> impl IntoView {
    let refresh = create_rw_signal(0);
    let rsc = create_resource(move || refresh.get(), move |_| get_hub_data());

    view! {
        <div class="p-6 grow h-full w-full">
            <h2 class="text-xl font-semibold">Dashboard</h2>
            <div class="my-1 flex justify-between">
                <p class="text-sm text-muted">"Provides pertinent system and alert information."</p>
                <button
                    type="button"
                    on:click=move |_| refresh.update(|v| *v += 1)
                    class="shrink-0 text-muted hover:text-primary pointer-events-auto"
                >
                    <RefreshArrow class="size-5" />
                </button>
            </div>

            <Transition fallback=move || {
                view! {
                    <div class="w-full h-64 flex items-center justify-center">
                        <SpinCircle class="animate-spin size-10" />
                    </div>
                }
            }>
                {move || {
                    if let Some(Ok(rsp)) = rsc.get() {
                        view! { <Overview data=rsp /> }
                    } else {
                        ().into_view()
                    }
                }}

            </Transition>
        </div>
    }
}

// ==================== // Overview // ==================== //

#[server]
async fn clean_shared_files() -> Result<String, ServerFnError<Error>> {
    use common::{AuthExtractor, ConfigExtractor, FileManager, StoreExtractor};

    let store = StoreExtractor::use_store()?;
    let _ = AuthExtractor::use_admin(false, &store).await?;

    let config = ConfigExtractor::use_config()?;
    FileManager::clean_outdated_files(config)
        .await
        .map_err(|err| err.into())
}

#[component]
fn Overview(data: HubData) -> impl IntoView {
    let HubData {
        num_feeds,
        feeds,
        num_users,
        users,
        num_clients,
        share_size,
    } = data;

    let toast = expect_context::<Toast>();
    let action = create_server_action::<CleanSharedFiles>();
    let value = action.value();
    create_effect(move |_| {
        value.with(|val| match val {
            Some(Err(FnError::WrappedServerError(e))) => toast.error(e.to_string()),
            Some(Ok(deleted_size)) => {
                toast.success(format!("{} has been clean", deleted_size));
            }
            _ => {}
        });
    });

    view! {
        <div class="mt-8 grid grid-cols-4 gap-5">
            <div class="p-5 rounded-md border border-border shadow-sm">
                <h4 class="mb-2 text-sm font-medium">"Active Chats"</h4>
                <p class="text-2xl font-bold">"#" {num_feeds}</p>
            </div>
            <div class="p-5 rounded-md border border-border shadow-sm">
                <h4 class="mb-2 text-sm font-medium">"Online Users"</h4>
                <p class="text-2xl font-bold">"#" {num_users}</p>
            </div>
            <div class="p-5 rounded-md border border-border shadow-sm">
                <h4 class="mb-2 text-sm font-medium">"Online Clients"</h4>
                <p class="text-2xl font-bold">"#" {num_clients}</p>
            </div>
            <div class="p-5 rounded-md border border-border shadow-sm">
                <div class="flex items-center justify-between">
                    <h4 class="mb-2 text-sm font-medium">"Shared files"</h4>
                    <ActionForm action>
                        <button type="submit" class="text-muted hover:text-primary">
                            <DeleteTrash class="size-4" />
                        </button>
                    </ActionForm>
                </div>
                <p class="text-xl font-bold">{share_size}</p>
            </div>
        </div>

        <div class="mt-12 grid grid-cols-8 gap-5">
            <div class="col-span-5 p-4 rounded-md border border-border shadow-sm">
                <h3 class="font-semibold">Chats</h3>
                <p class="text-sm font-medium text-muted">"Active feeds in the hub"</p>
                <FeedsTable feeds />
            </div>
            <div class="col-span-3 rounded-md border border-border shadow-sm">
                <div class="p-4">
                    <h3 class="font-semibold">Users</h3>
                    <p class="text-sm font-medium text-muted">"Current online users"</p>
                </div>
                <UsersSection users />
            </div>
        </div>
    }
}

// ==================== // FeedsTable // ==================== //

#[component]
fn FeedsTable(feeds: Vec<FeedData>) -> impl IntoView {
    let len = feeds.len() as i32;
    let feeds = store_value(feeds);
    let dts = expect_context::<DateTimeState>();

    view! {
        <table class="w-full mt-6 text-sm">
            <thead>
                <tr class="border-b border-border text-left text-muted hover:bg-accent/50">
                    <th class="h-10 px-2 font-medium">Feed</th>
                    <th class="h-10 px-2 font-medium">"Clients"</th>
                    <th class="h-10 px-2 font-medium">"Active"</th>
                </tr>
            </thead>
            <tbody>
                <For
                    each=move || feeds.get_value()
                    key=move |feed| feed.name.clone()
                    children=move |feed| {
                        let name = feed.name.split_once(":").map(|(_, name)| name.to_owned());
                        view! {
                            <tr class="last:border-b-0 border-b border-border hover:bg-accent/50">
                                <td class="px-2 h-12">{name}</td>
                                <td class="px-2 h-12">{feed.num_clients}</td>
                                <td class="px-2 h-12">{dts.fmt_sm(feed.active_at)}</td>
                            </tr>
                        }
                    }
                />

                {(len..5)
                    .into_iter()
                    .map(|_| {
                        view! {
                            <tr class="hover:bg-accent/50">
                                <td class="px-2 h-12 w-full" colspan="3"></td>
                            </tr>
                        }
                    })
                    .collect_view()}

            </tbody>
        </table>
    }
}

// ==================== // UsersSection // ==================== //

#[component]
fn UsersSection(users: Vec<User>) -> impl IntoView {
    let users = store_value(users);
    view! {
        <div class="px-6 py-4 grid gap-4">
            <For
                each=move || users.get_value()
                key=move |item| item.id
                children=move |user| {
                    view! {
                        <div class="flex items-center gap-3">
                            <Avatar src=user.avatar />
                            <div class="grid text-sm">
                                <p class="font-medium">{user.nickname}</p>
                                <p class="text-muted">"@" {user.username}</p>
                            </div>
                            <div class="ml-auto">
                                <UserRoleBadge role=user.role />
                            </div>
                        </div>
                    }
                }
            />

        </div>
    }
}
