use std::{cmp::Ordering, collections::HashMap, str::FromStr};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::map_res,
    multi::many0,
    sequence::{separated_pair, terminated},
    IResult,
};

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!(part_1 = ?part_1());
    tracing::info!(part_2 = ?part_2());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rule(u32, u32);

fn parse_number(input: &str) -> Result<(&str, u32), nom::Err<nom::error::Error<&str>>> {
    map_res(digit1, str::parse::<u32>)(input)
}

fn parse_rule(input: &str) -> IResult<&str, Rule> {
    let (rem, res) = separated_pair(parse_number, tag("|"), parse_number)(input)?;
    Ok((rem, Rule(res.0, res.1)))
}

impl FromStr for Rule {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, rule) = parse_rule(s).map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(rule)
    }
}

fn parse_printed_updates(input: &str) -> IResult<&str, Vec<u32>> {
    many0(alt((terminated(parse_number, tag(",")), parse_number)))(input)
}

trait Updates {
    fn parse_updates(input: &str) -> anyhow::Result<Self>
    where
        Self: Sized;
}

impl Updates for Vec<u32> {
    fn parse_updates(input: &str) -> anyhow::Result<Self> {
        parse_printed_updates(input)
            .map(|(_, updates)| updates)
            .map_err(|e| anyhow::anyhow!("{}", e))
    }
}

pub struct State {
    updates: Vec<Vec<u32>>,
    rules: Vec<Rule>,
    rule_map: HashMap<u32, Vec<Rule>>,
    update_map: Vec<HashMap<u32, usize>>,
}

fn get_problem_state() -> anyhow::Result<State> {
    let input = include_str!("../../inputs/day-5.txt");
    let &[rules, updates, ..] = input.split("\n\n").collect::<Vec<_>>().as_slice() else {
        anyhow::bail!("Invalid input");
    };
    let rules = rules
        .lines()
        .map(Rule::from_str)
        .collect::<anyhow::Result<Vec<_>>>()?;
    let updates = updates
        .lines()
        .map(Vec::<u32>::parse_updates)
        .collect::<anyhow::Result<Vec<_>>>()?;
    let rule_map = rules
        .clone()
        .into_iter()
        .flat_map(|Rule(a, b)| {
            let a = (
                a,
                rules
                    .clone()
                    .into_iter()
                    .filter(move |Rule(x, y)| *x == a || *y == a)
                    .collect(),
            );
            let b = (
                b,
                rules
                    .clone()
                    .into_iter()
                    .filter(move |Rule(x, y)| *x == b || *y == b)
                    .collect(),
            );
            std::iter::once(a).chain(std::iter::once(b))
        })
        .collect::<HashMap<u32, Vec<Rule>>>();
    let update_map: Vec<HashMap<_, _>> = updates
        .iter()
        .map(|update| update.iter().enumerate().map(|(i, el)| (*el, i)).collect())
        .collect();
    Ok(State {
        rules,
        updates,
        update_map,
        rule_map,
    })
}

fn ordered_correctly(
    el: u32,
    rule_map: &HashMap<u32, Vec<Rule>>,
    update_map: &HashMap<u32, usize>,
) -> bool {
    let Some(required_rules) = rule_map.get(&el) else {
        return true;
    };
    required_rules.iter().all(|Rule(a, b)| {
        let a_position = update_map.get(a);
        let b_position = update_map.get(b);
        match (a_position, b_position) {
            (Some(a), Some(b)) => a < b,
            _ => true,
        }
    })
}

fn updates_ordered(
    updates: &[u32],
    update_map: &HashMap<u32, usize>,
    rule_map: &HashMap<u32, Vec<Rule>>,
) -> bool {
    updates
        .iter()
        .all(|el| ordered_correctly(*el, rule_map, update_map))
}

fn part_1() -> anyhow::Result<u32> {
    let State {
        updates,
        update_map,
        rule_map,
        ..
    } = get_problem_state()?;
    let sum = updates
        .iter()
        .zip(update_map.iter())
        .filter(|(updates, update_map)| updates_ordered(updates, update_map, &rule_map))
        .map(|(updates, _)| {
            let middle = updates.len() / 2;
            updates[middle]
        })
        .sum();
    Ok(sum)
}

fn part_2() -> anyhow::Result<u32> {
    let State {
        mut updates,
        rules,
        rule_map,
        update_map,
    } = get_problem_state()?;
    let sum = updates
        .iter_mut()
        .zip(update_map.iter())
        .filter_map(|(updates, update_map)| {
            if updates_ordered(updates, update_map, &rule_map) {
                return None;
            }
            updates.sort_by(|a, b| {
                let rule = rules
                    .iter()
                    .find(|Rule(x, y)| { a == x && b == y } || { a == y && b == x });
                match rule {
                    None => Ordering::Equal,
                    Some(Rule(x, y)) if x == a && y == b => Ordering::Less,
                    Some(Rule(x, y)) if x == b && y == a => Ordering::Greater,
                    _ => unreachable!(),
                }
            });
            Some(updates)
        })
        .map(|updates| {
            let middle = updates.len() / 2;
            updates[middle]
        })
        .inspect(|middle| tracing::info!(?middle))
        .sum();
    Ok(sum)
}
