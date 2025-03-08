use leptos::*;
use leptos_dom::helpers::TimeoutHandle;
use std::time::Duration;

use super::icons::CloseXmark;

// ==================== // ToastContent // ==================== //

#[derive(Clone, Copy)]
#[repr(u8)]
enum ToastType {
    Info,
    Success,
    Error,
}

#[derive(Clone)]
struct ToastContent {
    variant: ToastType,
    message: String,
    handler: Option<TimeoutHandle>,
    show: RwSignal<bool>,
}

// ==================== // Toast // ==================== //

#[derive(Clone)]
pub struct Toast(StoredValue<ToastContent>);

impl Copy for Toast {}

impl Toast {
    pub fn new() -> Self {
        let content = ToastContent {
            variant: ToastType::Info,
            message: String::new(),
            handler: None,
            show: create_rw_signal(false),
        };
        Self(store_value(content))
    }

    pub fn success(&self, message: String) {
        self.add(ToastType::Success, message);
    }

    pub fn info(&self, message: String) {
        self.add(ToastType::Info, message);
    }

    pub fn error(&self, message: String) {
        self.add(ToastType::Error, message);
    }

    fn add(&self, variant: ToastType, message: String) {
        let show = self.show();
        if show.get_untracked() {
            self.close();
        }
        set_timeout(move || show.set(true), Duration::from_millis(160));

        let h = set_timeout_with_handle(move || show.set(false), Duration::from_secs(5)).ok();
        self.0.update_value(|v| {
            v.variant = variant;
            v.message = message;
            v.handler = h;
        });
    }

    fn close(&self) {
        self.0.update_value(|v| {
            if let Some(ref hander) = v.handler {
                hander.clear();
            }
            v.handler = None;
            v.show.set(false);
        })
    }

    fn show(&self) -> RwSignal<bool> {
        self.0.with_value(|v| v.show)
    }
}

// ==================== // Toaster // ==================== //

#[component]
pub fn Toaster() -> impl IntoView {
    let toast = expect_context::<Toast>();
    let show = toast.show();

    let cls = move || match toast.0.with_value(|v| v.variant) {
        ToastType::Info => {
            "rounded-md overflow-hidden py-3 px-4 bg-primary text-primary-on shadow-lg"
        }
        ToastType::Success => {
            "rounded-md overflow-hidden py-3 px-4 bg-success text-success-on shadow-lg"
        }
        ToastType::Error => {
            "rounded-md overflow-hidden py-3 px-4 bg-danger text-danger-on shadow-lg"
        }
    };

    let on_mouse_enter = move |_| {
        if show.get_untracked() {
            toast.0.update_value(|v| {
                if let Some(ref h) = v.handler {
                    h.clear();
                    v.handler = None;
                }
            });
        }
    };
    let on_mouse_leave = move |_| {
        let h = set_timeout_with_handle(move || show.set(false), Duration::from_secs(5)).ok();
        toast.0.update_value(|v| v.handler = h);
    };

    view! {
        <div class="fixed bottom-5 left-5 z-50">
            <AnimatedShow
                when=show
                show_class="animate-slide-in-up"
                hide_class="animate-slide-out-left"
                hide_delay=Duration::from_millis(150)
            >
                <div on:mouseenter=on_mouse_enter on:mouseleave=on_mouse_leave class=cls>
                    <div class="max-w-md min-w-48 w-fit flex items-center space-x-4 pointer-events-auto">
                        <p class="grow line-clamp-2 text-sm font-medium">{toast.0.with_value(|v| v.message.clone())}</p>
                        <button type="button" on:click=move |_| toast.close() class="shrink-0">
                            <CloseXmark class="size-4 stroke-2" />
                        </button>
                    </div>
                </div>
            </AnimatedShow>
        </div>
    }
}
