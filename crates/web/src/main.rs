mod api;
mod components;

use leptos::prelude::*;
use leptos::task::spawn_local;
use tracker_core::Dataset;

use components::dashboard::Dashboard;
use components::item_detail::ItemDetail;
use components::item_form::ItemForm;

/// Which screen is showing. Kept in a signal instead of a router to stay dependency-light.
#[derive(Clone, PartialEq)]
pub enum View {
    Dashboard,
    NewItem,
    Edit(String),
    Detail(String),
}

#[derive(Clone, Copy)]
pub struct AppCtx {
    pub data: RwSignal<Option<Dataset>>,
    pub view: RwSignal<View>,
}

impl AppCtx {
    /// Re-fetch the dataset from the server into the shared signal.
    pub fn refresh(self) {
        spawn_local(async move {
            match api::fetch_data().await {
                Ok(d) => self.data.set(Some(d)),
                Err(e) => leptos::logging::error!("failed to fetch data: {e}"),
            }
        });
    }
}

#[component]
fn App() -> impl IntoView {
    let ctx = AppCtx { data: RwSignal::new(None), view: RwSignal::new(View::Dashboard) };
    provide_context(ctx);
    ctx.refresh();

    view! {
        <header><h1 style="margin:0;font-size:18px;">"Asset Value Tracker"</h1></header>
        <main>
            {move || match ctx.view.get() {
                View::Dashboard => view! { <Dashboard/> }.into_any(),
                View::NewItem => view! { <ItemForm/> }.into_any(),
                View::Edit(id) => view! { <ItemForm id=id/> }.into_any(),
                View::Detail(id) => view! { <ItemDetail id=id/> }.into_any(),
            }}
        </main>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
