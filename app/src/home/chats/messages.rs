use leptos::*;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, Event, HtmlImageElement, MouseEvent};

use super::DateTimeState;
use crate::components::icons::FileDownload;
use crate::components::Avatar;
use crate::home::{ChatsState, UserState};
use common::{Message, MessageKind};

#[component]
pub fn Messages() -> impl IntoView {
    let chats = expect_context::<ChatsState>();
    let room_id = chats.room_id();
    let message_map = chats.messages();

    let messages =
        move || with!(|room_id, message_map| message_map.get(room_id).cloned().unwrap_or_default());

    let ul_ref = create_node_ref::<html::Ul>();
    let scroll_to_bottom = move || {
        if let Some(node) = ul_ref.get() {
            node.scroll_to_with_x_and_y(0.0, node.scroll_height() as f64);
        }
    };

    create_effect(move |_| {
        if with!(|room_id, message_map| message_map.get(room_id).is_some()) {
            scroll_to_bottom();
        }
    });

    let image = create_rw_signal(String::new());

    view! {
        <ul class="grow h-full scrollbar scrollbar-container" node_ref=ul_ref>
            <For
                each=messages
                key=|message| message.id
                children=move |message| {
                    view! { <MessageItem message image on_load=move |_| scroll_to_bottom() /> }
                }
            />

        </ul>

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
fn MessageItem<F>(message: Message, image: RwSignal<String>, on_load: F) -> impl IntoView
where
    F: Fn(Event) + 'static,
{
    let user = expect_context::<UserState>().get();
    let dts = expect_context::<DateTimeState>();

    let Message {
        content,
        url,
        kind,
        divide,
        sender,
        send_at,
        ..
    } = message;

    let incoming = move || user.with(|v| v.id != sender.id);

    let cbubble = move || {
        if incoming() {
            "mb-5 px-2 flex flex-row items-start gap-3"
        } else {
            "mb-5 px-2 flex flex-row-reverse items-start gap-3"
        }
    };

    let on_error = move |ev: ErrorEvent| {
        ev.prevent_default();
        let target = ev
            .current_target()
            .unwrap()
            .unchecked_into::<HtmlImageElement>();
        target.set_src("/default/fallback.png");
    };

    let on_click_view = move |ev: MouseEvent| {
        ev.prevent_default();
        let target = ev.target().unwrap().unchecked_into::<HtmlImageElement>();
        image.set(target.src());
    };

    view! {
        <Show when=move || divide>
            <li class="text-center my-5">
                <span class="text-sm text-muted">{move || dts.fmt_lg(send_at)}</span>
            </li>
        </Show>

        <li class=cbubble>
            <Avatar src=sender.avatar />
            {match kind {
                MessageKind::Text => {
                    let ctext = move || {
                        if incoming() {
                            "max-w-lg px-3 py-2 rounded-md shadow-sm bg-accent text-accent-on whitespace-pre-wrap"
                        } else {
                            "max-w-lg px-3 py-2 rounded-md shadow-sm bg-primary text-primary-on whitespace-pre-wrap"
                        }
                    };
                    view! { <div class=ctext>{content}</div> }
                }
                MessageKind::Image => {
                    view! {
                        <div class="max-w-72 rounded-md overflow-hidden bg-accent cursor-zoom-in">
                            <img
                                src=url
                                alt=content
                                on:load=on_load
                                on:error=on_error
                                on:click=on_click_view
                                class="object-cover object-center"
                            />
                        </div>
                    }
                }
                MessageKind::File => {
                    view! {
                        <div class="w-fit rounded-md bg-accent px-4 py-3">
                            <a href=url download=content.clone() class="group space-x-2">
                                <span class="text-sm text-accent-on group-hover:text-success">{content}</span>
                                <FileDownload class="inline-block size-6 fill-accent-on group-hover:fill-success" />
                            </a>
                        </div>
                    }
                }
            }}

        </li>
    }
}
