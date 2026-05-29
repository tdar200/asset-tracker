use leptos::prelude::*;
use tracker_core::Item;

use crate::components::chart::ValueChart;
use crate::{AppCtx, View};

#[component]
pub fn ItemDetail(id: String) -> impl IntoView {
    let ctx = expect_context::<AppCtx>();
    let id_for_lookup = id.clone();

    view! {
        <button on:click=move |_| ctx.view.set(View::Dashboard)>"← Back"</button>
        {move || {
            let item: Option<Item> = ctx.data.get()
                .and_then(|d| d.items.into_iter().find(|i| i.id == id_for_lookup));
            match item {
                None => view! { <p>"Item not found."</p> }.into_any(),
                Some(item) => {
                    let edit_id = item.id.clone();
                    view! {
                        <div style="display:flex;justify-content:space-between;align-items:center;">
                            <h2>{item.name.clone()}</h2>
                            <button on:click=move |_| ctx.view.set(View::Edit(edit_id.clone()))>"Edit"</button>
                        </div>
                        <p style="color:#54595f;">{item.details.clone()}" · "{item.condition.clone()}</p>
                        <div class="card" style="margin-bottom:16px;">
                            <ValueChart item=item.clone()/>
                        </div>
                        <div class="card">
                            <h3 style="margin-top:0;">"Valuation history"</h3>
                            <table>
                                <thead><tr><th>"Date"</th><th>"Low"</th><th>"Best"</th><th>"High"</th><th>"Confidence"</th><th>"Sources"</th></tr></thead>
                                <tbody>
                                    {item.valuations.iter().rev().map(|v| {
                                        let conf = format!("{:?}", v.confidence);
                                        view! {
                                            <tr>
                                                <td>{v.date.clone()}</td>
                                                <td>{format!("{:.0}", v.low)}</td>
                                                <td>{format!("{:.0}", v.best)}</td>
                                                <td>{format!("{:.0}", v.high)}</td>
                                                <td>{conf}</td>
                                                <td>{v.sources.iter().map(|s| view! {
                                                    <a href=s.url.clone() target="_blank" style="margin-right:6px;">{s.name.clone()}</a>
                                                }).collect_view()}</td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </div>
                    }.into_any()
                }
            }
        }}
    }
}
