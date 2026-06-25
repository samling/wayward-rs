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
