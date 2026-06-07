use relm4::gtk;
use relm4::gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use super::history::BatteryHistoryGraphPoint;

#[derive(Clone)]
pub(super) struct BatteryHistoryGraph {
    root: gtk::DrawingArea,
    points: Rc<RefCell<Vec<BatteryHistoryGraphPoint>>>,
}

impl BatteryHistoryGraph {
    pub(super) fn new() -> Self {
        let root = gtk::DrawingArea::new();
        root.add_css_class("battery-history-graph");
        root.set_content_width(260);
        root.set_content_height(96);

        let points = Rc::new(RefCell::new(Vec::<BatteryHistoryGraphPoint>::new()));
        let draw_points = points.clone();

        root.set_draw_func(move |_area, context, width, height| {
            draw_graph(context, width as f64, height as f64, &draw_points.borrow());
        });

        Self { root, points }
    }

    pub(super) fn root(&self) -> gtk::DrawingArea {
        self.root.clone()
    }

    pub(super) fn set_points(&self, points: Vec<BatteryHistoryGraphPoint>) {
        self.points.replace(points);
        self.root.queue_draw();
    }
}

fn draw_graph(
    context: &gtk::cairo::Context,
    width: f64,
    height: f64,
    points: &[BatteryHistoryGraphPoint],
) {
    if width <= 0.0 || height <= 0.0 {
        return;
    }

    let padding_left = 34.0;
    let padding_right = 8.0;
    let padding_top = 10.0;
    let padding_bottom = 20.0;

    let graph_x = padding_left;
    let graph_y = padding_top;
    let graph_width = (width - padding_left - padding_right).max(0.0);
    let graph_height = (height - padding_top - padding_bottom).max(0.0);

    draw_background(context, graph_x, graph_y, graph_width, graph_height);
    draw_y_markers(context, graph_x, graph_y, graph_width, graph_height);
    draw_axis_labels(context, width, height, padding_left, padding_top, padding_bottom);

    if points.len() < 2 {
        return;
    }

    context.set_source_rgba(137.0 / 255.0, 180.0 / 255.0, 250.0 / 255.0, 0.9);
    context.set_line_width(2.0);

    for (index, point) in points.iter().enumerate() {
        let x = graph_x + point.x * graph_width;
        let y = graph_y + graph_height - point.y * graph_height;

        if index == 0 {
            context.move_to(x, y);
        } else {
            context.line_to(x, y);
        }
    }

    let _ = context.stroke();
}

fn draw_background(context: &gtk::cairo::Context, x: f64, y: f64, width: f64, height: f64) {
    context.set_source_rgba(241.0 / 255.0, 243.0 / 255.0, 244.0 / 255.0, 0.06);
    context.rectangle(x, y, width, height);
    let _ = context.fill();
}

fn draw_y_markers(
    context: &gtk::cairo::Context,
    graph_x: f64,
    graph_y: f64,
    graph_width: f64,
    graph_height: f64,
) {
    context.select_font_face(
        "JetBrainsMono",
        gtk::cairo::FontSlant::Normal,
        gtk::cairo::FontWeight::Normal,
    );
    context.set_font_size(9.0);

    for (label, percent) in [("75%", 0.75), ("50%", 0.5), ("25%", 0.25)] {
        let y = graph_y + graph_height - percent * graph_height;

        context.set_source_rgba(241.0 / 255.0, 243.0 / 255.0, 244.0 / 255.0, 0.12);
        context.set_line_width(1.0);
        context.move_to(graph_x, y);
        context.line_to(graph_x + graph_width, y);
        let _ = context.stroke();

        context.set_source_rgba(241.0 / 255.0, 243.0 / 255.0, 244.0 / 255.0, 0.5);
        context.move_to(6.0, y + 3.0);
        let _ = context.show_text(label);
    }

}

fn draw_axis_labels(
    context: &gtk::cairo::Context,
    width: f64,
    height: f64,
    padding_left: f64,
    padding_top: f64,
    padding_bottom: f64,
) {
    context.set_source_rgba(241.0 / 255.0, 243.0 / 255.0, 244.0 / 255.0, 0.62);
    context.select_font_face("JetBrainsMono", gtk::cairo::FontSlant::Normal, gtk::cairo::FontWeight::Normal);
    context.set_font_size(9.0);

    context.move_to(2.0, padding_top + 7.0);
    let _ = context.show_text("100%");

    context.move_to(8.0, height - padding_bottom);
    let _ = context.show_text("0%");

    context.move_to(padding_left, height - 4.0);
    let _ = context.show_text("8h ago");

    let now = "Now";
    let x = context
        .text_extents(now)
        .map(|extents| width - extents.width() - 2.0)
        .unwrap_or(width - 24.0);

    context.move_to(x, height - 4.0);
    let _ = context.show_text(now);
}