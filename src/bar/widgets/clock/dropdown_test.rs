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
