use chrono;
use relm4::Sender;
use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use relm4::gtk::prelude::WidgetExt;
use std::time::Duration;

use crate::bar::dropdown::Dropdown;
use crate::bar::layout::BarEdge;
use crate::bar::state::{BarItemState, ClockState};
use crate::bar::widget::BarWidgetRuntime;
use crate::bar::widget::{BarWidget, WidgetInstance};
use crate::bar::{BarContext, BarMsg};
use crate::shell::ShellMsg;

pub(crate) struct ClockWidget;

struct ClockRuntime {
    root: gtk::MenuButton,
    label: gtk::Label,
    dropdown: Dropdown,
    format: String,
    edge: std::rc::Rc<std::cell::Cell<BarEdge>>,
}

impl BarWidgetRuntime for ClockRuntime {
    fn root(&self) -> gtk::Widget {
        self.root.clone().upcast()
    }

    fn update(&mut self, state: &BarItemState, context: &BarContext) {
        let BarItemState::Clock(ClockState::Ready) = state else {
            return;
        };

        self.edge.set(context.edge);
        self.dropdown.set_edge(context.edge);

        self.label
            .set_text(&chrono::Local::now().format(&self.format).to_string());
    }
}

impl BarWidget for ClockWidget {
    fn id(&self) -> &'static str {
        "clock"
    }

    fn build(
        &self,
        instance: &WidgetInstance,
        _sender: &relm4::Sender<BarMsg>,
        _services: &crate::services::ShellServices,
    ) -> Box<dyn BarWidgetRuntime> {
        let format = instance
            .config
            .get("format")
            .and_then(|value| value.as_str())
            .unwrap_or("%H:%M")
            .to_string();

        let label = gtk::Label::new(Some(&chrono::Local::now().format(&format).to_string()));

        label.add_css_class("clock-label");

        let edge = std::rc::Rc::new(std::cell::Cell::new(BarEdge::Top));
        let child = calendar_dropdown_content(chrono::Local::now().date_naive());
        let (root, dropdown) = Dropdown::menu_button("clock", BarEdge::Top, &label, &child);

        dropdown.bind_to_menu_button(&root, BarEdge::Top, &child);

        Box::new(ClockRuntime {
            root,
            label,
            dropdown,
            format,
            edge,
        })
    }

    fn initial_state(&self) -> Option<BarItemState> {
        Some(BarItemState::Clock(ClockState::Ready))
    }

    fn start(
        &self,
        sender: relm4::Sender<ShellMsg>,
        _services: &crate::services::ShellServices,
    ) -> Option<relm4::JoinHandle<()>> {
        Some(start(sender))
    }
}

pub(crate) fn start(sender: relm4::Sender<ShellMsg>) -> relm4::tokio::task::JoinHandle<()> {
    relm4::spawn(async move {
        run_clock(sender).await;
    })
}

pub(super) async fn run_clock(sender: Sender<ShellMsg>) {
    loop {
        relm4::tokio::time::sleep(Duration::from_secs(1)).await;

        if sender.send(clock_message()).is_err() {
            return;
        }
    }
}

fn clock_message() -> ShellMsg {
    ShellMsg::ItemStateChanged(BarItemState::Clock(ClockState::Ready))
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CalendarDay {
    date: chrono::NaiveDate,
    is_current_month: bool,
    is_today: bool,
}

fn month_grid(today: chrono::NaiveDate) -> Vec<CalendarDay> {
    use chrono::{Datelike, Duration};

    let first_of_month = today.with_day(1).unwrap_or(today);
    let month = first_of_month.month();
    let leading_days = first_of_month.weekday().num_days_from_sunday();
    let grid_start = first_of_month - Duration::days(i64::from(leading_days));

    (0..42)
        .map(|index| {
            let date = grid_start + Duration::days(index);
            CalendarDay {
                date,
                is_current_month: date.month() == month,
                is_today: date == today,
            }
        })
        .collect()
}

fn calendar_dropdown_content(today: chrono::NaiveDate) -> gtk::Box {
    use chrono::Datelike;
    use relm4::gtk::prelude::{BoxExt, GridExt, WidgetExt};

    let root = gtk::Box::new(gtk::Orientation::Vertical, 8);
    root.add_css_class("dropdown-content");
    root.add_css_class("clock-dropdown-content");

    let title = gtk::Label::new(Some(&today.format("%B %Y").to_string()));
    title.add_css_class("dropdown-title");
    title.add_css_class("clock-dropdown-title");
    root.append(&title);

    let grid = gtk::Grid::new();
    grid.add_css_class("clock-calendar");
    grid.set_row_spacing(4);
    grid.set_column_spacing(4);

    for (column, label) in ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]
        .iter()
        .enumerate()
    {
        let day_label = gtk::Label::new(Some(label));
        day_label.add_css_class("clock-calendar-weekday");
        grid.attach(&day_label, column as i32, 0, 1, 1);
    }

    for (index, day) in month_grid(today).iter().enumerate() {
        let label = gtk::Label::new(Some(&day.date.day().to_string()));
        label.add_css_class("clock-calendar-day");

        if !day.is_current_month {
            label.add_css_class("other-month");
        }

        if day.is_today {
            label.add_css_class("today");
        }

        let row = (index / 7) as i32 + 1;
        let column = (index % 7) as i32;

        grid.attach(&label, column, row, 1, 1);
    }

    root.append(&grid);
    root
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn month_grid_always_has_six_weeks() {
        let days = month_grid(date(2026, 6, 2));

        assert_eq!(days.len(), 42);
    }

    #[test]
    fn month_grid_includes_leading_days() {
        let days = month_grid(date(2026, 8, 15));

        assert_eq!(days[0].date, date(2026, 7, 26));
        assert!(!days[0].is_current_month);
        assert_eq!(days[6].date, date(2026, 8, 1));
        assert!(days[6].is_current_month);
    }

    #[test]
    fn month_grid_marks_today() {
        let today = date(2026, 6, 2);
        let days = month_grid(today);

        let today_cell = days.iter().find(|day| day.date == today).unwrap();

        assert!(today_cell.is_today);
        assert!(today_cell.is_current_month);
    }
}
