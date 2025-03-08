use leptos::*;
use std::time::Duration;

use super::{RtcStatus, WebRtcState, WebSocketState};
use crate::components::icons::{MicOff, MicOn, PhoneSolid};
use crate::components::{Avatar, Toast};
use crate::home::ChatsState;
use common::HungUpReson;

#[component]
pub fn CallSection() -> impl IntoView {
    let toast = expect_context::<Toast>();
    let chats = expect_context::<ChatsState>();
    let ws = expect_context::<WebSocketState>();
    let rtc = expect_context::<WebRtcState>();
    let status = rtc.status();

    let on_hungup = move |_| {
        let reson = match status.get_untracked() {
            RtcStatus::Caller => HungUpReson::Cancel,
            RtcStatus::Callee => HungUpReson::Refuse,
            RtcStatus::Calling | RtcStatus::Idle => HungUpReson::Finish,
        };
        rtc.send_hung_up(reson, ws);
    };

    let friend = create_memo(move |_| {
        if status.get() == RtcStatus::Idle {
            None
        } else {
            if let Some(friend_id) = rtc.friend_id() {
                chats
                    .friends()
                    .with_untracked(|v| v.iter().find(|v| v.id == friend_id).cloned())
            } else {
                None
            }
        }
    });

    let get_src = move || friend.with(|v| v.as_ref().map(|x| x.avatar.clone()).unwrap_or_default());
    let get_nickname =
        move || friend.with(|v| v.as_ref().map(|x| x.nickname.clone()).unwrap_or_default());

    let muted = create_rw_signal(!rtc.has_audio());

    view! {
        <div class="fixed inset-x-0 top-4 flex justify-center">
            <AnimatedShow
                when=Signal::derive(move || status.get() != RtcStatus::Idle)
                show_class="animate-slide-in-down"
                hide_class="animate-slide-out-up"
                hide_delay=Duration::from_millis(150)
            >
                <div class="flex items-center gap-3 rounded-md px-4 py-2 bg-surface text-surface-on border border-border shadow-sm">
                    <Avatar src=Signal::derive(get_src) />
                    <div class="text-sm pr-2">
                        <p class="text-surface-on font-semibold text-center">{get_nickname}</p>
                        <div class="flex items-center space-x-2">
                            <Show
                                when=move || status.get() == RtcStatus::Calling
                                fallback=|| {
                                    view! {
                                        <span class="relative flex size-2">
                                            <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-success opacity-75"></span>
                                            <span class="relative inline-flex rounded-full size-2 bg-success"></span>
                                        </span>
                                        <p class="text-xs text-success">Waiting</p>
                                    }
                                }
                            >
                                <span class="relative inline-flex rounded-full size-2 bg-success"></span>
                                <p class="text-xs text-success">Speaking</p>
                            </Show>
                        </div>
                    </div>
                    <div>
                        <audio id="pcaudio" autoplay=true></audio>
                    </div>
                    <Show
                        when=move || status.get() == RtcStatus::Calling
                        fallback=move || {
                            view! {
                                <button
                                    type="button"
                                    on:click=move |_| rtc.send_reply(ws)
                                    disabled=move || status.get() == RtcStatus::Caller
                                    class="rounded-full p-2 bg-success text-success-on disabled:bg-muted"
                                >
                                    <PhoneSolid class="size-4" />
                                </button>
                            }
                        }
                    >
                        <button
                            type="button"
                            on:click=move |_| rtc.toggle_mute(muted, toast)
                            class="rounded-full p-2 bg-accent text-accent-on border border-border"
                        >
                            <Show when=move || muted.get() fallback=|| view! { <MicOn class="size-4" /> }>
                                <MicOff class="size-4" />
                            </Show>
                        </button>
                    </Show>
                    <button type="button" on:click=on_hungup class="rounded-full p-2 bg-danger text-danger-on">
                        <PhoneSolid class="size-4 origin-center rotate-[135deg] translate-y-0.5" />
                    </button>
                </div>
            </AnimatedShow>
        </div>
    }
}
