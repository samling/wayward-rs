use crate::bar::{dropdown, layout::BarEdge, widget::BarRegion};
use chrono::NaiveDate;
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};

pub(super) struct ClockDropdown {
    date: NaiveDate,
    edge: BarEdge,
    region: BarRegion,
}

pub(super) struct ClockDropdownInit {
    pub(super) date: NaiveDate,
    pub(super) edge: BarEdge,
    pub(super) region: BarRegion,
}

#[derive(Debug)]
pub(super) enum ClockDropdownInput {
    SetDate(NaiveDate),
    SetPlacement { edge: BarEdge, region: BarRegion },
}

pub(super) struct ClockDropdownWidgets {
    popover: gtk::Popover,
    revealer: gtk::Revealer,
    title: gtk::Label,
    day_labels: Vec<gtk::Label>,
}

impl SimpleComponent for ClockDropdown {
    type Init = ClockDropdownInit;
    type Input = ClockDropdownInput;
    type Output = ();
    type Root = gtk::Popover;
    type Widgets = ClockDropdownWidgets;

    fn init_root() -> Self::Root {
        let root = gtk::Popover::new();
        root.set_has_arrow(false);
        root.set_autohide(true);
        root.add_css_class("dropdown");
        root.add_css_class("clock-dropdown");
        root
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            date: init.date,
            edge: init.edge,
            region: init.region,
        };

        let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
        content.add_css_class("dropdown-content");
        content.add_css_class("clock-dropdown-content");

        let title = gtk::Label::new(Some(&model.date.format("%B %Y").to_string()));
        title.add_css_class("dropdown-header");
        title.add_css_class("dropdown-title");
        title.add_css_class("clock-dropdown-title");
        content.append(&title);

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

        let day_labels = month_grid(model.date)
            .iter()
            .enumerate()
            .map(|(index, day)| {
                let label = gtk::Label::new(None);
                label.add_css_class("clock-calendar-day");
                update_day_label(&label, day);

                let row = (index / 7) as i32 + 1;
                let column = (index % 7) as i32;
                grid.attach(&label, column, row, 1, 1);

                label
            })
            .collect();

        content.append(&grid);
        root.set_child(Some(&content));

        let revealer = gtk::Revealer::new();
        dropdown::install_revealer(&root, &revealer, model.edge, model.region);

        let widgets = ClockDropdownWidgets {
            popover: root,
            revealer,
            title,
            day_labels,
        };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            ClockDropdownInput::SetDate(date) => {
                self.date = date;
            }
            ClockDropdownInput::SetPlacement { edge, region } => {
                self.edge = edge;
                self.region = region;
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        dropdown::set_placement(&widgets.popover, &widgets.revealer, self.edge, self.region);

        widgets
            .title
            .set_text(&self.date.format("%B %Y").to_string());

        for (label, day) in widgets.day_labels.iter().zip(month_grid(self.date)) {
            update_day_label(label, &day);
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CalendarDay {
    date: NaiveDate,
    is_current_month: bool,
    is_today: bool,
}

fn month_grid(today: NaiveDate) -> Vec<CalendarDay> {
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

fn update_day_label(label: &gtk::Label, day: &CalendarDay) {
    use chrono::Datelike;

    label.set_text(&day.date.day().to_string());
    label.remove_css_class("other-month");
    label.remove_css_class("today");

    if !day.is_current_month {
        label.add_css_class("other-month");
    }

    if day.is_today {
        label.add_css_class("today");
    }
}

#[cfg(test)]
mod tests {
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
