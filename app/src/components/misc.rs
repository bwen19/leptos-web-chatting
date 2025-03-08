use leptos::*;
use std::time::Duration;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, HtmlImageElement};

use super::icons::SelectArrow;
use common::UserRole;

// ==================== // Avatar // ==================== //

#[component]
pub fn Avatar(
    #[prop(into)] src: MaybeSignal<String>,
    #[prop(into, default = "size-10".into())] size: MaybeSignal<String>,
) -> impl IntoView {
    let wrapper = move || {
        size.with(|sz| {
            format!(
                "flex shrink-0 overflow-hidden rounded-full bg-accent {}",
                sz
            )
        })
    };

    let on_error = move |ev: ErrorEvent| {
        ev.prevent_default();
        let target = ev
            .current_target()
            .unwrap()
            .unchecked_into::<HtmlImageElement>();
        target.set_src("/default/anon.png");
    };

    let src = move || {
        if src.with(String::is_empty) {
            String::from("/default/anon.png")
        } else {
            src.get()
        }
    };

    view! {
        <div class=wrapper>
            <img alt="User avatar" src=src on:error=on_error class="w-full h-full aspect-square" role="img" />
        </div>
    }
}

// ==================== // MenuListItem // ==================== //

#[component]
pub fn MenuListItem(
    #[prop(default = false.into(), into)] active: MaybeSignal<bool>,
    children: Children,
) -> impl IntoView {
    let cls = move || {
        if active.get() {
            "w-full rounded-lg px-4 py-3 mb-2 flex items-center gap-3 text-accent-on font-medium bg-accent cursor-pointer"
        } else {
            "w-full rounded-lg px-4 py-3 mb-2 flex items-center gap-3 text-muted hover:text-accent-on hover:bg-accent cursor-pointer"
        }
    };
    view! { <li class=cls>{children()}</li> }
}

// ==================== // ModalWrapper // ==================== //

#[component]
pub fn ModalWrapper(children: Children) -> impl IntoView {
    view! {
        <div class="fixed inset-0 z-20 min-h-full flex items-center justify-center bg-container/60 backdrop-blur-sm">
            <div class="w-96 rounded-lg bg-surface text-surface-on border border-border shadow-sm flex flex-col items-center p-6">
                {children()}
            </div>
        </div>
    }
}

// ==================== // UserRoleBadge // ==================== //

#[component]
pub fn UserRoleBadge(role: UserRole) -> impl IntoView {
    match role {
        UserRole::Admin => view! {
            <div class="inline-flex items-center rounded-md px-2.5 py-0.5 bg-danger/90 text-danger-on text-xs font-semibold">
                Admin
            </div>
        },
        UserRole::User => view! {
            <div class="inline-flex items-center rounded-md px-2.5 py-0.5 bg-success/90 text-success-on text-xs font-semibold">
                User
            </div>
        },
    }
}

// ==================== // UserActiveBadge // ==================== //

#[component]
pub fn UserActiveBadge(active: bool) -> impl IntoView {
    if active {
        view! {
            <div class="inline-flex items-center rounded-md px-2.5 py-0.5 border border-border text-primary text-xs font-semibold">
                Active
            </div>
        }
    } else {
        view! {
            <div class="inline-flex items-center rounded-md px-2 py-0.5 border border-transparent bg-accent text-muted text-xs font-semibold">
                Inactive
            </div>
        }
    }
}

// ==================== // UserActiveBadge // ==================== //

pub trait SelectLabel {
    fn label(&self) -> String;
}

impl SelectLabel for bool {
    fn label(&self) -> String {
        match self {
            true => String::from("True"),
            false => String::from("False"),
        }
    }
}

impl SelectLabel for UserRole {
    fn label(&self) -> String {
        match self {
            UserRole::Admin => String::from("Admin"),
            UserRole::User => String::from("User"),
        }
    }
}

#[component]
pub fn Selector<T>(value: RwSignal<T>, options: StoredValue<Vec<T>>) -> impl IntoView
where
    T: SelectLabel + Clone + 'static,
{
    let show_menu = create_rw_signal(false);
    let on_select = move |val: T| {
        value.set(val);
        show_menu.set(false);
    };

    let section_ref = create_node_ref::<html::Div>();
    use_click_outside(section_ref, Callback::new(move |_| show_menu.set(false)));

    view! {
        <div class="relative" node_ref=section_ref>
            <button type="button" on:click=move |_| show_menu.update(|v| *v = !*v) class="w-full h-9 px-3 btn-select">
                <span class="pointer-events-none">{move || value.with(|v| v.label())}</span>
                <SelectArrow class="h-4 w-4 opacity-50" />
            </button>
            <AnimatedShow
                when=show_menu
                show_class="animate-fade-in"
                hide_class="animate-fade-out"
                hide_delay=Duration::from_millis(150)
            >
                <ul class="max-h-96 absolute z-40 inset-x-0 top-10 overflow-hidden p-1 rounded-md border border-border bg-surface shadow-md">
                    {options
                        .get_value()
                        .into_iter()
                        .map(|item| {
                            let item = store_value(item);
                            view! {
                                <li
                                    on:click=move |_| on_select(item.get_value())
                                    class="py-1 px-2 rounded-md text-sm cursor-default hover:bg-accent"
                                >
                                    {item.with_value(|v| v.label())}
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            </AnimatedShow>
        </div>
    }
}

// ==================== // use_click_outside // ==================== //

#[component]
pub fn BlankTableItem(
    #[prop(default = String::new().into(), into)] msg: MaybeSignal<String>,
    #[prop(default = 1.into(), into)] cols: MaybeSignal<i32>,
) -> impl IntoView {
    view! {
        <tr>
            <td class="px-2 h-14 w-full" colspan=cols.get()>
                <p class="text-center font-semibold text-muted">{msg}</p>
            </td>
        </tr>
    }
}

#[component]
pub fn BlankTable(
    #[prop(default = String::new().into(), into)] msg: MaybeSignal<String>,
    #[prop(default = 1.into(), into)] rows: MaybeSignal<i32>,
    #[prop(default = 1.into(), into)] cols: MaybeSignal<i32>,
) -> impl IntoView {
    view! {
        <BlankTableItem msg=msg.clone() cols=cols />
        {move || { (1..rows.get()).into_iter().map(|_| view! { <BlankTableItem cols=cols /> }).collect::<Vec<_>>() }}
    }
}

// ==================== // use_click_outside // ==================== //

pub fn use_click_outside(element: NodeRef<html::Div>, on_click: Callback<()>) {
    cfg_if::cfg_if! { if #[cfg(feature = "hydrate")] {
        let handle = window_event_listener(ev::click, move |ev| {
            use leptos::wasm_bindgen::__rt::IntoJsResult;
            let el = ev.target();
            let mut el: Option<web_sys::Element> =
                el.into_js_result().map_or(None, |el| Some(el.into()));
            let body = document().body().unwrap();
            while let Some(current_el) = el {
                if current_el == *body {
                    break;
                };
                let Some(displayed_el) = element.get_untracked() else {
                    break;
                };
                if current_el == ***displayed_el {
                    return;
                }
                el = current_el.parent_element();
            }
            on_click.call(());
        });
        on_cleanup(move || handle.remove());
    } else {
        let _ = element;
        let _ = on_click;
    }}
}
