use leptos::prelude::*;
use tracker_core::{gain_loss, totals_by_currency, Item};

use crate::{AppCtx, View};

#[component]
pub fn Dashboard() -> impl IntoView {
    let ctx = expect_context::<AppCtx>();

    view! {
        <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:14px;">
            <h2 style="margin:0;">"Holdings"</h2>
            <button class="primary" on:click=move |_| ctx.view.set(View::NewItem)>"+ Add item"</button>
        </div>
        {move || match ctx.data.get() {
            None => view! { <p>"Loading…"</p> }.into_any(),
            Some(data) if data.items.is_empty() => view! { <p>"No items yet. Add your first asset."</p> }.into_any(),
            Some(data) => {
                let totals = totals_by_currency(&data.items);
                view! {
                    <div class="card" style="margin-bottom:16px;">
                        <h3 style="margin-top:0;">"Portfolio (by currency)"</h3>
                        <table>
                            <thead><tr><th>"Currency"</th><th>"Cost"</th><th>"Current"</th><th>"Gain/Loss"</th></tr></thead>
                            <tbody>
                                {totals.into_iter().map(|t| {
                                    let cls = if t.total_gain >= 0.0 { "gain-pos" } else { "gain-neg" };
                                    view! {
                                        <tr>
                                            <td>{t.currency.clone()}</td>
                                            <td>{format!("{:.2}", t.total_cost)}</td>
                                            <td>{format!("{:.2}", t.total_current)}</td>
                                            <td class=cls>{format!("{:+.2}", t.total_gain)}</td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    </div>
                    <div class="grid">
                        {data.items.into_iter().map(|item| view! { <ItemCard item=item/> }).collect_view()}
                    </div>
                }.into_any()
            }
        }}
    }
}

#[component]
fn ItemCard(item: Item) -> impl IntoView {
    let ctx = expect_context::<AppCtx>();
    let id = item.id.clone();
    let current = item.current_best();
    let (gl_text, gl_cls) = match current {
        Some(c) => {
            let g = gain_loss(item.purchase_price, c);
            (format!("{:+.2} ({:+.1}%)", g.absolute, g.percent),
             if g.absolute >= 0.0 { "gain-pos" } else { "gain-neg" })
        }
        None => ("no valuation yet".to_string(), ""),
    };
    let cur_text = current.map(|c| format!("{} {:.2}", item.currency, c))
        .unwrap_or_else(|| "—".into());

    view! {
        <div class="card" style="cursor:pointer;" on:click=move |_| ctx.view.set(View::Detail(id.clone()))>
            <strong>{item.name.clone()}</strong>
            <div style="font-size:12px;color:#54595f;">{item.category.clone()}</div>
            <p style="margin:8px 0 2px;">"Current: "{cur_text}</p>
            <p style="margin:2px 0;font-size:13px;">"Paid: "{format!("{} {:.2}", item.currency, item.purchase_price)}</p>
            <p class=gl_cls style="margin:2px 0;">{gl_text}</p>
        </div>
    }
}
