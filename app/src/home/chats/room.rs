use leptos::*;
use server_fn::codec::{MultipartData, MultipartFormData};
use wasm_bindgen::JsCast;
use web_sys::{File, FormData, HtmlFormElement};

use super::{emoji::EmojiButton, messages::Messages};
use crate::components::icons::{
    AirPlane, CallPhone, ChatBubble, DeleteTrash, FileUpload, PlusCircle, UploadArrow,
};
use crate::components::Toast;
use crate::connection::{RtcStatus, WebRtcState, WebSocketState};
use crate::home::{ChatsState, UserState};
use common::{Error, Event, FileInfo, FileMeta, FnError, Message};

#[component]
pub fn RoomPage() -> impl IntoView {
    let chats = expect_context::<ChatsState>();
    let room_id = chats.room_id();
    let rooms = chats.rooms();

    let ws = expect_context::<WebSocketState>();
    let rtc = expect_context::<WebRtcState>();
    let status = rtc.status();

    let room_pos = create_memo(move |_| {
        with!(|room_id, rooms| { rooms.iter().position(|v| v.id.as_str() == room_id) })
    });

    create_effect(move |_| {
        if let Some(pos) = room_pos.get() {
            rooms.update(|rms| {
                if let Some(room) = rms.get_mut(pos) {
                    let room_unreads = room.unreads;
                    if room_unreads > 0 {
                        room.check();
                        chats.unreads().update(|x| *x -= room_unreads);
                    }
                }
            })
        }
    });

    let room_name = Signal::derive(move || {
        if let Some(pos) = room_pos.get() {
            rooms.with_untracked(|v| v.get(pos).map(|x| Some(x.name.to_owned())).unwrap_or(None))
        } else {
            None
        }
    });

    let on_click = move |_| {
        let room_id = room_id.get_untracked();
        let friend_id = chats.friends().with_untracked(|fds| {
            let friend = fds.iter().find(|v| v.room_id.as_str() == room_id);
            friend.map(|v| v.id).unwrap_or(0)
        });
        if friend_id > 0 {
            rtc.send_call(friend_id, ws);
        }
    };

    view! {
        <Show
            when=move || room_name.with(Option::is_some)
            fallback=|| {
                view! {
                    <div class="h-full w-full flex items-center justify-center">
                        <ChatBubble class="size-32 stroke-accent" />
                    </div>
                }
            }
        >

            <div class="h-full w-full flex flex-col">
                <div class="shrink-0 px-6 py-4 border-b border-border flex items-center justify-between">
                    <p class="font-medium text-lg">{move || room_name.get().unwrap_or(String::new())}</p>
                    <button
                        on:click=on_click
                        disabled=move || status.get() != RtcStatus::Idle
                        class="text-muted hover:text-primary disabled:text-muted"
                    >
                        <CallPhone class="size-5" />
                    </button>
                </div>
                <Messages />
                <ChatBar />
            </div>
        </Show>
    }
}

// ========================================================= //

#[server(input = MultipartFormData)]
pub async fn upload_file(data: MultipartData) -> Result<FileMeta, ServerFnError<Error>> {
    use common::{AuthExtractor, ConfigExtractor, FileManager, StoreExtractor};

    let store = StoreExtractor::use_store()?;
    let _ = AuthExtractor::use_auth(false, &store).await?;

    // `.into_inner()` returns the inner `multer` stream, it is `None` if we call this on the client,
    // but always `Some(_)` on the server, so is safe to unwrap
    let mut data = data.into_inner().unwrap();

    if let Ok(Some(field)) = data.next_field().await {
        let config = ConfigExtractor::use_config()?;
        let rsp = FileManager::save_shared_file(field.into(), config).await?;
        Ok(rsp)
    } else {
        Err(Error::BadRequest(String::from("No field found in multipart data")).into())
    }
}

#[component]
fn ChatBar() -> impl IntoView {
    let toast = expect_context::<Toast>();
    let ws = expect_context::<WebSocketState>();
    let chats = expect_context::<ChatsState>();
    let user = expect_context::<UserState>().get();

    // handler for send text message
    let text_ref = create_node_ref::<html::Textarea>();
    let content = create_rw_signal(String::new());

    create_effect(move |_| {
        if !chats.room_id().with(String::is_empty) {
            if let Some(el) = text_ref.get_untracked() {
                let _ = el.focus();
            }
        }
    });

    let send_text_message = move || {
        if !content.with(String::is_empty) {
            let msg = Message::text(
                chats.room_id().get_untracked(),
                user.get_untracked(),
                content.get_untracked(),
            );
            ws.send(Event::Send(msg));
            content.set(String::new());
        }
    };

    let on_click_send = move |_| send_text_message();
    let on_ctrl_enter = move |ev: ev::KeyboardEvent| {
        if ev.ctrl_key() && ev.key() == "Enter" {
            send_text_message()
        }
    };

    // handle file info
    let input_ref = create_node_ref::<html::Input>();
    let has_file = create_rw_signal(false);
    let file_info = create_rw_signal(FileInfo::default());

    let add_file = move |file: File| {
        file_info.set(FileInfo::new(file.name(), file.size()));
        has_file.set(true);
    };
    let del_file = move || {
        file_info.set(FileInfo::default());
        has_file.set(false);
    };

    let on_change = move |_| {
        if let Some(input_ref) = input_ref.get_untracked() {
            if let Some(files) = input_ref.files() {
                if let Some(file) = files.item(0) {
                    add_file(file);
                }
            }
        }
    };

    // handle drag event
    let has_drag = create_rw_signal(false);

    let on_drop = move |ev: ev::DragEvent| {
        ev.prevent_default();
        if let Some(data) = ev.data_transfer() {
            if let Some(ref files) = data.files() {
                if let Some(input_ref) = input_ref.get_untracked() {
                    input_ref.set_files(Some(files));
                    if let Some(file) = files.item(0) {
                        add_file(file);
                        has_drag.set(false);
                    }
                }
            }
        }
    };
    let on_dragover = move |ev: ev::DragEvent| {
        ev.prevent_default();
        has_drag.set(true);
    };
    let on_dragenter = move |ev: ev::DragEvent| {
        ev.prevent_default();
    };
    let on_dragleave = move |ev: ev::DragEvent| {
        ev.prevent_default();
        has_drag.set(false);
    };

    // handler action of uploading file
    let action = create_action(|data: &FormData| {
        let data = data.clone();
        upload_file(data.into())
    });
    let pending = action.pending();

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        let target = ev.target().unwrap().unchecked_into::<HtmlFormElement>();
        let form_data = FormData::new_with_form(&target).unwrap();
        action.dispatch(form_data);
        target.reset();
        del_file();
    };

    let on_reset = move |_| del_file();

    // automatically send ws message when uploading successfully
    create_effect(move |_| {
        action.value().with(|val| match val {
            Some(Err(FnError::WrappedServerError(e))) => toast.error(e.to_string()),
            Some(Ok(file_meta)) => {
                let msg = Message::file(
                    chats.room_id().get_untracked(),
                    user.get_untracked(),
                    file_meta.to_owned(),
                );
                ws.send(Event::Send(msg));
            }
            _ => {}
        });
    });

    view! {
        <div
            on:drop=on_drop
            on:dragover=on_dragover
            on:dragenter=on_dragenter
            on:dragleave=on_dragleave
            class="shrink-0 h-16 w-full py-2 flex items-center"
            class:bg-muted=has_drag
        >
            <label for="upload" class="inline-block mx-3 rounded-full p-1 cursor-pointer">
                <PlusCircle class="size-6 stroke-muted hover:stroke-surface-on" />
            </label>
            <form
                on:submit=on_submit
                on:reset=on_reset
                class="grow mr-6 px-5 h-10 flex items-center rounded-md bg-accent border border-border"
                class:hidden=move || !has_file.get()
            >
                <input
                    id="upload"
                    type="file"
                    name="file_to_upload"
                    on:change=on_change
                    node_ref=input_ref
                    class="hidden"
                />
                <FileUpload class="fill-danger size-5" />
                <p class="mx-2 max-w-md text-accent-on truncate">{move || file_info.with(|fl| fl.name.to_owned())}</p>
                <p class="grow shrink-0 text-muted">{move || file_info.with(|fl| fl.size.to_owned())}</p>
                <button
                    type="reset"
                    disabled=pending
                    class="mx-4 shrink-0 text-muted hover:text-primary pointer-events-auto"
                >
                    <DeleteTrash class="size-5" />
                </button>
                <button
                    type="submit"
                    disabled=pending
                    class="shrink-0 text-muted hover:text-primary pointer-events-auto"
                >
                    <UploadArrow class="size-5" />
                </button>
            </form>
            <div class="grow flex items-center" class:hidden=has_file>
                <div class="grow relative">
                    <textarea
                        rows="1"
                        placeholder="Type message here and send it by CTRL+Enter"
                        spellcheck=false
                        autocomplete="off"
                        on:input=move |ev| content.set(event_target_value(&ev))
                        on:keydown=on_ctrl_enter
                        prop:value=content
                        node_ref=text_ref
                        class="block w-full h-10 pt-2 pl-4 pr-10 scrollbar resize-none input"
                    ></textarea>
                    <div class="absolute right-3 top-2">
                        <EmojiButton content />
                    </div>
                </div>
                <button type="button" on:click=on_click_send class="rounded-full mx-3 p-1 flex">
                    <AirPlane class="size-6 -rotate-45 -translate-y-0.5 fill-primary hover:fill-success" />
                </button>
            </div>
        </div>
    }
}
