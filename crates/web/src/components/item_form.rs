use leptos::prelude::*;

#[component]
pub fn ItemForm(#[prop(optional)] id: Option<String>) -> impl IntoView {
    let _ = id;
    view! { <p>"form placeholder"</p> }
}
