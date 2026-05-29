mod api;
mod components;

use leptos::prelude::*;
use components::dashboard::Dashboard;

#[component]
fn App() -> impl IntoView {
    view! {
        <header><h1 style="margin:0;font-size:18px;">"Asset Value Tracker"</h1></header>
        <main><Dashboard/></main>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
