#![feature(iter_map_windows)]

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!(part_1 = part_1());
    tracing::info!(part_2 = part_2());
}

fn find_report_error(report: &[i32]) -> Option<usize> {
    let mut diffs = report
        .iter()
        .map_windows(|[&a, &b]| b - a)
        .map(|diff| (diff.signum(), matches!(diff.abs(), 1..=3)));
    let first = diffs.next()?;
    diffs.position(|d| d != first)
}

fn part_1() -> usize {
    let input = include_str!("../../inputs/day-2.txt");
    input
        .lines()
        .filter(|line| {
            let report = line
                .split(" ")
                .flat_map(|s| s.parse::<i32>())
                .collect::<Vec<_>>();
            find_report_error(&report).is_none()
        })
        .count()
}

fn part_2() -> usize {
    let input = include_str!("../../inputs/day-2.txt");
    input
        .lines()
        .filter(|line| {
            let report: Vec<_> = line.split(" ").flat_map(|s| s.parse::<i32>()).collect();
            let Some(error) = find_report_error(&report) else {
                return true;
            };
            for i in 0..=1 {
                let error = error + i;
                let report = [&report[..error], &report[(error + 1)..]].concat();
                if find_report_error(&report).is_none() {
                    return true;
                }
            }
            false
        })
        .count()
}
