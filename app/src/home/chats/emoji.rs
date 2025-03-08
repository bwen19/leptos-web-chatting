use leptos::*;
use std::time::Duration;

use crate::components::icons::SmileEmoji;
use crate::components::use_click_outside;

#[component]
pub fn EmojiButton(content: RwSignal<String>) -> impl IntoView {
    let emoji_data = vec![
        "😄", "😃", "😉", "😏", "😩", "😘", "😜", "😭", "👿", "👹", "👺", "👽", "💀", "🙈", "🙉",
        "🙊", "👍", "👎", "👌", "👊", "✊", "👋", "✋", "👆", "👇", "👈", "👉", "🙌", "🙏", "👏",
        "💪", "🚶", "🏃", "💃", "👫", "👯", "👶", "💥", "💯", "💢", "💫", "✨", "🔥", "💦", "❄️",
        "⚡", "🌀", "💤", "👸", "🍔", "🧀", "🥪", "🥚", "🍱", "🍘", "🥟", "🦀", "🦞", "🍉", "🍈",
        "🍇", "🍊", "🍎", "🥭", "🍍", "🍌", "🍐", "🍓", "🍅", "🥔", "🥕", "🥜", "🍄", "🐀", "🐂",
        "🐉", "🐅", "🐇", "🐓", "🐟", "🐬", "🐋", "🐕", "🐖", "🐧", "🐢", "🐯", "🐨", "😺", "😸",
        "🍃", "🌹", "🍁", "⛅", "🌔", "🌍", "🌜", "🔎", "🔍", "📧", "🔨", "💰", "📮", "🎺", "🎵",
        "🎹", "🎺", "🎻", "🥁", "📱", "☎️", "📞", "💻", "💡", "🔦", "📕", "📖", "📚", "📄", "📰",
        "💵", "💴", "💶", "💳", "📧", "✏️", "📈", "🔑", "🔒", "🔓", "🏹", "🔫", "🔬", "🧬", "🧹",
        "🚆", "🚇", "🚌", "🚕", "🚑", "🚒", "🛵", "🚲", "✈️", "🚢", "🚀", "🚁", "⏱", "☀️", "🌛",
        "🔥", "💧", "🎄", "🎆", "🏆", "🏅", "🥇", "🥈", "🥉", "⚽", "⚾", "🏀", "🎾", "🏸", "🏓",
    ];

    let show_popover = create_rw_signal(false);
    let target_ref = create_node_ref::<html::Div>();

    let on_select = move |emo: &str| {
        content.update(|v| *v += emo);
        show_popover.set(false);
    };

    use_click_outside(target_ref, Callback::new(move |_| show_popover.set(false)));

    view! {
        <div node_ref=target_ref class="relative">
            <SmileEmoji
                on:click=move |_| show_popover.update(|v| *v = !*v)
                class="size-6 stroke-muted hover:stroke-surface-on cursor-pointer"
            />
            <AnimatedShow
                when=show_popover
                show_class="animate-fade-in"
                hide_class="animate-fade-out"
                hide_delay=Duration::from_millis(150)
            >
                <div class="absolute z-30 -right-4 bottom-7 h-96 min-w-max rounded-md grid grid-cols-8 scrollbar scrollbar-container bg-surface border border-border shadow-md">
                    {emoji_data
                        .iter()
                        .map(|v| {
                            let emo = v.to_string();
                            view! {
                                <button
                                    type="button"
                                    on:click=move |_| on_select(&emo)
                                    class="p-1.5 rounded-full text-center text-lg hover:bg-accent"
                                >
                                    {v.to_string()}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
            </AnimatedShow>
        </div>
    }
}
