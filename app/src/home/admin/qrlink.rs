use leptos::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlButtonElement, MouseEvent};

use super::qrlink_ops::{DeleteLinkButton, UploadFileButton};
use crate::components::icons::SpinCircle;
use crate::components::{BlankTable, BlankTableItem};
use common::{Error, FileLink, FileLinks};

#[server]
async fn list_links(page_id: i32) -> Result<FileLinks, ServerFnError<Error>> {
    use common::{AuthExtractor, StoreExtractor};

    let store = StoreExtractor::use_store()?;
    let _ = AuthExtractor::use_admin(false, &store).await?;

    let rsp = FileLink::list(page_id, &store).await?;
    Ok(rsp)
}

#[component]
pub fn QrLinkPage() -> impl IntoView {
    let refresh = create_rw_signal(0);
    let total = create_rw_signal(0);
    let pages = create_rw_signal(0);
    let page_id = create_rw_signal(1);
    let image = create_rw_signal(String::new());

    let links_resource = create_resource(
        move || (refresh.get(), page_id.get()),
        move |(_, page_id)| list_links(page_id),
    );

    let tab_content = move || {
        links_resource
            .get()
            .map(|res: Result<FileLinks, ServerFnError<Error>>| match res {
                Ok(rsp) => {
                    total.set(rsp.total);
                    pages.set(rsp.pages);

                    let mut vlist = rsp
                        .filelinks
                        .into_iter()
                        .map(|filelink| view! { <LinksTableItem filelink refresh image /> })
                        .collect::<Vec<_>>();
                    let start = vlist.len() as i32;
                    for _ in start..5 {
                        vlist.push(view! { <BlankTableItem cols=4 /> });
                    }
                    vlist.into_view()
                }
                Err(e) => view! { <BlankTable rows=5 cols=4 msg=e.to_string() /> },
            })
            .unwrap_or_default()
    };

    view! {
        <div class="p-6 grow h-full w-full">
            <h2 class="text-xl font-semibold">Qrlink</h2>
            <div class="my-1 flex justify-between">
                <p class="text-sm text-muted">"Manage QR links for file download."</p>
                <UploadFileButton refresh />
            </div>

            <div class="w-full mt-8 overflow-auto rounded-md border border-border">
                <table class="w-full text-sm">
                    <thead>
                        <tr class="border-b border-border text-left text-muted hover:bg-accent/50">
                            <th class="h-10 px-4 font-medium">Name</th>
                            <th class="h-10 px-2 font-medium">File</th>
                            <th class="h-10 px-3 font-medium">QR</th>
                            <th class="h-10 px-2"></th>
                        </tr>
                    </thead>
                    <tbody>
                        <Transition fallback=|| {
                            view! {
                                <div class="w-full h-56 flex items-center justify-center">
                                    <SpinCircle class="animate-spin size-10" />
                                </div>
                            }
                        }>{tab_content}</Transition>
                    </tbody>
                </table>
            </div>
        </div>

        <Show when=move || !image.with(String::is_empty)>
            <Portal mount=document().get_element_by_id("app").unwrap()>
                <div
                    on:click=move |_| image.set(String::new())
                    class="fixed inset-0 z-40 h-screen flex items-center justify-center bg-container/95"
                >
                    <div class="max-w-5xl bg-accent rounded-lg shadow-sm border border-border overflow-hidden">
                        <img src=image alt="View image" />
                    </div>
                </div>
            </Portal>
        </Show>
    }
}

#[component]
fn LinksTableItem(
    filelink: FileLink,
    refresh: RwSignal<i32>,
    image: RwSignal<String>,
) -> impl IntoView {
    let filelink = store_value(filelink);

    let on_click_view = move |ev: MouseEvent| {
        ev.prevent_default();
        let target = ev.target().unwrap().unchecked_into::<HtmlButtonElement>();
        image.set(target.value());
    };

    view! {
        <tr class="last:border-b-0 border-b border-border hover:bg-accent/50">
            <td class="px-4 h-12">{filelink.with_value(|v| v.name.clone())}</td>
            <td class="px-2 h-12">
                <a
                    href=filelink.with_value(|v| v.link.clone())
                    download=filelink.with_value(|v| v.name.clone())
                    class="text-sm text-accent-on hover:text-success underline"
                >
                    "Download"
                </a>
            </td>
            <td class="px-2 h-12">
                <button
                    type="button"
                    value=filelink.with_value(|v| v.qrlink.clone())
                    on:click=on_click_view
                    class="px-3 h-6 btn-primary"
                >
                    "Show QR"
                </button>
            </td>
            <td class="pr-6 h-12">
                <div class="flex items-center justify-center">
                    <DeleteLinkButton link_id=filelink.with_value(|v| v.id) refresh />
                </div>
            </td>
        </tr>
    }
}
