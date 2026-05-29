use leptos::prelude::*;
use leptos::task::spawn_local;
use tracker_core::{Item, NewItem};

use crate::{api, AppCtx, View};

#[component]
pub fn ItemForm(#[prop(optional)] id: Option<String>) -> impl IntoView {
    let ctx = expect_context::<AppCtx>();

    // Seed fields from the existing item when editing.
    let existing: Option<Item> = id.as_ref().and_then(|id| {
        ctx.data.get().and_then(|d| d.items.into_iter().find(|i| &i.id == id))
    });

    let name = RwSignal::new(existing.as_ref().map(|i| i.name.clone()).unwrap_or_default());
    let category = RwSignal::new(existing.as_ref().map(|i| i.category.clone()).unwrap_or_default());
    let details = RwSignal::new(existing.as_ref().map(|i| i.details.clone()).unwrap_or_default());
    let condition = RwSignal::new(existing.as_ref().map(|i| i.condition.clone()).unwrap_or_default());
    let currency = RwSignal::new(existing.as_ref().map(|i| i.currency.clone()).unwrap_or_else(|| "GBP".into()));
    let price = RwSignal::new(existing.as_ref().map(|i| i.purchase_price).unwrap_or(0.0));
    let date = RwSignal::new(existing.as_ref().map(|i| i.purchase_date.clone()).unwrap_or_default());
    let notes = RwSignal::new(existing.as_ref().map(|i| i.notes.clone()).unwrap_or_default());
    let error = RwSignal::new(String::new());

    let editing_item = existing.clone();
    let on_save = move |_| {
        let ctx = ctx;
        let editing_item = editing_item.clone();
        let payload = NewItem {
            name: name.get(), category: category.get(), details: details.get(),
            condition: condition.get(), currency: currency.get(),
            purchase_price: price.get(), purchase_date: date.get(), notes: notes.get(),
        };
        spawn_local(async move {
            let result = match editing_item {
                Some(mut item) => {
                    item.name = payload.name; item.category = payload.category;
                    item.details = payload.details; item.condition = payload.condition;
                    item.currency = payload.currency; item.purchase_price = payload.purchase_price;
                    item.purchase_date = payload.purchase_date; item.notes = payload.notes;
                    api::update_item(&item).await
                }
                None => api::create_item(&payload).await.map(|_| ()),
            };
            match result {
                Ok(()) => { ctx.refresh(); ctx.view.set(View::Dashboard); }
                Err(e) => error.set(e),
            }
        });
    };

    let delete_id = id.clone();
    let on_delete = move |_| {
        if let Some(id) = delete_id.clone() {
            let ctx = ctx;
            spawn_local(async move {
                match api::delete_item(&id).await {
                    Ok(()) => { ctx.refresh(); ctx.view.set(View::Dashboard); }
                    Err(e) => error.set(e),
                }
            });
        }
    };

    let is_edit = id.is_some();

    view! {
        <div class="card" style="max-width:520px;">
            <h2 style="margin-top:0;">{move || if is_edit { "Edit item" } else { "Add item" }}</h2>
            <label>"Name"</label>
            <input prop:value=move || name.get() on:input=move |e| name.set(event_target_value(&e))/>
            <label>"Category"</label>
            <input prop:value=move || category.get() on:input=move |e| category.set(event_target_value(&e))/>
            <label>"Details (model no., specs — helps valuation accuracy)"</label>
            <input prop:value=move || details.get() on:input=move |e| details.set(event_target_value(&e))/>
            <label>"Condition"</label>
            <input prop:value=move || condition.get() on:input=move |e| condition.set(event_target_value(&e))/>
            <label>"Currency (e.g. GBP, USD)"</label>
            <input prop:value=move || currency.get() on:input=move |e| currency.set(event_target_value(&e))/>
            <label>"Purchase price"</label>
            <input type="number" step="0.01" prop:value=move || price.get()
                on:input=move |e| price.set(event_target_value(&e).parse().unwrap_or(0.0))/>
            <label>"Purchase date (YYYY-MM-DD)"</label>
            <input type="date" prop:value=move || date.get() on:input=move |e| date.set(event_target_value(&e))/>
            <label>"Notes"</label>
            <textarea on:input=move |e| notes.set(event_target_value(&e))>{move || notes.get()}</textarea>
            <p style="color:#b5341a;">{move || error.get()}</p>
            <div style="display:flex;gap:8px;margin-top:12px;">
                <button class="primary" on:click=on_save>"Save"</button>
                <button on:click=move |_| ctx.view.set(View::Dashboard)>"Cancel"</button>
                {is_edit.then(|| view! { <button on:click=on_delete style="margin-left:auto;color:#b5341a;">"Delete"</button> })}
            </div>
        </div>
    }
}
