use std::collections::HashMap;

use anyhow::Context;
use nom::{
    character::complete::{digit1, multispace1},
    combinator::map_res,
    sequence::tuple,
    IResult,
};

pub fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let input = include_str!("../../inputs/day-1.txt");
    let res = input
        .lines()
        .map(|line| parse_line(line))
        .inspect(|result| {
            tracing::info!(?result);
        })
        .collect::<Result<Vec<_>, _>>()?;
    tracing::info!(part_1 = part_1(&res));
    tracing::info!(part_2 = part_2(&res));
    Ok(())
}

fn part_1(lines: &[(&str, (u32, u32))]) -> u32 {
    let mut left: Vec<_> = lines.iter().map(|(_, (a, _))| *a).collect();
    let mut right: Vec<_> = lines.iter().map(|(_, (_, b))| *b).collect();
    left.sort_unstable();
    right.sort_unstable();
    assert!(left.len() == right.len());
    left.iter()
        .zip(right.iter())
        .map(|(a, b)| if a < b { b - a } else { a - b })
        .sum::<u32>()
}

fn part_2(lines: &[(&str, (u32, u32))]) -> u32 {
    let right: HashMap<u32, u32> =
        lines
            .iter()
            .map(|(_, (_, b))| *b)
            .fold(HashMap::default(), |mut acc, el| {
                acc.entry(el).and_modify(|v| *v += 1).or_insert(1);
                acc
            });
    lines
        .iter()
        .map(|(_, (a, _))| *a)
        .map(|a| right.get(&a).cloned().unwrap_or_default() * a)
        .sum()
}

fn parse_line(line: &str) -> IResult<&str, (u32, u32)> {
    map_res(
        tuple((digit1, multispace1, digit1)),
        |(a, _, b): (&str, &str, &str)| {
            a.parse()
                .ok()
                .zip(b.parse().ok())
                .context("Could not parse line")
        },
    )(line)
}
