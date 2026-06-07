use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const UPOWER_HISTORY_DIR: &str = "/var/lib/upower";
const CHARGE_HISTORY_PREFIX: &str = "history-charge-";
const CHARGE_HISTORY_SUFFIX: &str = ".dat";
pub(super) const CHARGE_HISTORY_WINDOW_SECONDS: i64 = 8 * 60 * 60; // 8 hours

#[derive(Clone, Debug, PartialEq)]
pub(super) struct BatteryHistoryPoint {
    pub(super) timestamp: i64,
    pub(super) percentage: f64,
    pub(super) state: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct BatteryHistoryGraphPoint {
    pub(super) x: f64,
    pub(super) y: f64,
}

pub(super) fn load_charge_history() -> io::Result<Vec<BatteryHistoryPoint>> {
    let Some(path) = charge_history_path()? else {
        return Ok(Vec::new());
    };

    let contents = fs::read_to_string(path)?;
    Ok(parse_charge_history(&contents))
}

fn charge_history_path() -> io::Result<Option<PathBuf>> {
    let mut candidates = Vec::new();

    for entry in fs::read_dir(UPOWER_HISTORY_DIR)? {
        let entry = entry?;
        let path = entry.path();

        if !is_charge_history_file(&path) {
            continue;
        }

        let size = entry.metadata()?.len();
        candidates.push((size, path));
    }

    candidates.sort_by_key(|(size, _)| *size);

    Ok(candidates.pop().map(|(_, path)| path))
}

fn is_charge_history_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| {
            name.starts_with(CHARGE_HISTORY_PREFIX) && name.ends_with(CHARGE_HISTORY_SUFFIX)
        })
        .unwrap_or(false)
}

pub(super) fn parse_charge_history(input: &str) -> Vec<BatteryHistoryPoint> {
    input
        .lines()
        .filter_map(parse_charge_history_line)
        .collect()
}

pub(super) fn graph_points(points: &[BatteryHistoryPoint]) -> Vec<BatteryHistoryGraphPoint> {
    let Some(first) = points.first() else {
        return Vec::new();
    };

    let Some(last) = points.last() else {
        return Vec::new();
    };

    let duration = (last.timestamp - first.timestamp).max(1) as f64;

    points
        .iter()
        .map(|point| BatteryHistoryGraphPoint {
            x: (point.timestamp - first.timestamp) as f64 / duration,
            y: (point.percentage / 100.0).clamp(0.0, 1.0),
        })
        .collect()
}

pub(super) fn recent_points(
    points: &[BatteryHistoryPoint],
    window_seconds: i64,
) -> Vec<BatteryHistoryPoint> {
    let Some(last) = points.last() else {
        return Vec::new();
    };

    let cutoff = last.timestamp - window_seconds;

    points
        .iter()
        .filter(|point| point.timestamp >= cutoff)
        .cloned()
        .collect()
}

fn parse_charge_history_line(line: &str) -> Option<BatteryHistoryPoint> {
    let mut fields = line.split_whitespace();

    let timestamp = fields.next()?.parse().ok()?;
    let percentage = fields.next()?.parse().ok()?;
    let state = fields.next()?.to_string();

    Some(BatteryHistoryPoint { timestamp, percentage, state })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_charge_history_reads_upower_rows() {
        let input = "\
1780776698\t82.000\tdischarging
1780776998\t81.000\tdischarging
";

        let points = parse_charge_history(input);

        assert_eq!(
            points,
            vec![
                BatteryHistoryPoint {
                    timestamp: 1780776698,
                    percentage: 82.0,
                    state: "discharging".to_string(),
                },
                BatteryHistoryPoint {
                    timestamp: 1780776998,
                    percentage: 81.0,
                    state: "discharging".to_string(),
                },
            ]
        );
    }
}

#[test]
fn graph_points_normalize_time_and_percentage() {
    let points = vec![
        BatteryHistoryPoint {
            timestamp: 100,
            percentage: 25.0,
            state: "discharging".to_string(),
        },
        BatteryHistoryPoint {
            timestamp: 150,
            percentage: 50.0,
            state: "discharging".to_string(),
        },
        BatteryHistoryPoint {
            timestamp: 200,
            percentage: 75.0,
            state: "charging".to_string(),
        },
    ];

    let graph = graph_points(&points);

    assert_eq!(
        graph,
        vec![
            BatteryHistoryGraphPoint { x: 0.0, y: 0.25 },
            BatteryHistoryGraphPoint { x: 0.5, y: 0.5 },
            BatteryHistoryGraphPoint { x: 1.0, y: 0.75 },
        ]
    );
}