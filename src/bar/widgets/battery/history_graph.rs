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
        root.set_content_width(240);
        root.set_content_height(72);

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

    draw_background(context, width, height);

    if points.len() < 2 {
        return;
    }

    context.set_source_rgba(137.0 / 255.0, 180.0 / 255.0, 250.0 / 255.0, 0.9);
    context.set_line_width(2.0);

    for (index, point) in points.iter().enumerate() {
        let x = point.x * width;
        let y = height - point.y * height;

        if index == 0 {
            context.move_to(x, y);
        } else {
            context.line_to(x, y);
        }
    }

    let _ = context.stroke();
}

fn draw_background(context: &gtk::cairo::Context, width: f64, height: f64) {
    context.set_source_rgba(241.0 / 255.0, 243.0 / 255.0, 244.0 / 255.0, 0.06);
    context.rectangle(0.0, 0.0, width, height);
    let _ = context.fill();
}