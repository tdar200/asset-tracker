use leptos::html::Canvas;
use leptos::prelude::*;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use tracker_core::Item;
use web_sys::HtmlCanvasElement;

/// Value-over-time chart: shaded low–high band, best-estimate line, purchase-price anchor.
#[component]
pub fn ValueChart(item: Item) -> impl IntoView {
    let canvas_ref: NodeRef<Canvas> = NodeRef::new();

    // Redraw whenever the canvas mounts.
    Effect::new(move |_| {
        if let Some(canvas) = canvas_ref.get() {
            let el: HtmlCanvasElement = canvas.into();
            draw(&el, &item);
        }
    });

    view! { <canvas node_ref=canvas_ref width="680" height="340"></canvas> }
}

fn draw(canvas: &HtmlCanvasElement, item: &Item) {
    let backend = match CanvasBackend::with_canvas_object(canvas.clone()) {
        Some(b) => b,
        None => return,
    };
    let area = backend.into_drawing_area();
    area.fill(&WHITE).ok();

    if item.valuations.is_empty() {
        return;
    }

    let n = item.valuations.len();
    let xs: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let highs: Vec<f64> = item.valuations.iter().map(|v| v.high).collect();
    let lows: Vec<f64> = item.valuations.iter().map(|v| v.low).collect();
    let bests: Vec<f64> = item.valuations.iter().map(|v| v.best).collect();

    let y_min = lows.iter().cloned().fold(item.purchase_price, f64::min) * 0.95;
    let y_max = highs.iter().cloned().fold(item.purchase_price, f64::max) * 1.05;
    let x_max = if n > 1 { (n - 1) as f64 } else { 1.0 };

    let mut chart = ChartBuilder::on(&area)
        .margin(10)
        .x_label_area_size(36)
        .y_label_area_size(52)
        .build_cartesian_2d(0f64..x_max, y_min..y_max)
        .unwrap();

    chart
        .configure_mesh()
        .x_labels(n.min(8))
        .x_label_formatter(&|x| {
            let i = x.round() as usize;
            item.valuations.get(i).map(|v| v.date.clone()).unwrap_or_default()
        })
        .draw()
        .ok();

    // Low–high band as a filled polygon (high forward, low back).
    let mut band: Vec<(f64, f64)> = xs.iter().zip(&highs).map(|(&x, &h)| (x, h)).collect();
    band.extend(xs.iter().zip(&lows).rev().map(|(&x, &l)| (x, l)));
    chart
        .draw_series(std::iter::once(Polygon::new(band, BLUE.mix(0.12))))
        .ok();

    // Best-estimate line.
    chart
        .draw_series(LineSeries::new(xs.iter().zip(&bests).map(|(&x, &b)| (x, b)), BLUE.stroke_width(2)))
        .ok();

    // Purchase-price reference line.
    chart
        .draw_series(LineSeries::new(vec![(0.0, item.purchase_price), (x_max, item.purchase_price)], RED.mix(0.6)))
        .ok();

    area.present().ok();
}
