#![feature(iter_map_windows)]
use std::{collections::HashMap, ops::Div};

use anyhow::{anyhow, bail, Context};
use indicatif::ProgressStyle;
use nom::{
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{map, map_res},
    multi::separated_list1,
    sequence::separated_pair,
    IResult,
};
use tailcall::tailcall;
use tracing::{span, Level, Span};
use tracing_indicatif::{span_ext::IndicatifSpanExt, IndicatifLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn main() -> anyhow::Result<()> {
    let indicatif_layer = IndicatifLayer::new();

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive("info".parse()?))
        .with(tracing_subscriber::fmt::layer().with_writer(indicatif_layer.get_stderr_writer()))
        .with(indicatif_layer)
        .init();
    tracing::info!(part_1 = ?part_1());
    tracing::info!(part_2 = ?part_2());
    Ok(())
}

#[derive(Debug)]
struct Numbers {
    result: u64,
    numbers: Vec<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Operator {
    Add,
    Multiply,
    Concat,
}

impl Operator {
    fn apply(&self, a: u64, b: u64) -> u64 {
        match self {
            Self::Add => a + b,
            Self::Multiply => a * b,
            Self::Concat => (a.to_string() + &b.to_string())
                .parse()
                .expect("Invalid concatenation"),
        }
    }
}

#[derive(Debug)]
struct Equation<'a> {
    numbers: &'a Numbers,
    operators: &'a [Operator],
}

impl Equation<'_> {
    fn calculate(&self) -> anyhow::Result<u64> {
        let first = *self.numbers.numbers.first().context("No numbers")?;
        let res = self
            .numbers
            .numbers
            .iter()
            .skip(1)
            .zip(self.operators.iter())
            .fold(first, |a, (b, op)| op.apply(a, *b));
        Ok(res)
    }
}

fn solve_both(allowed: &[Operator]) -> anyhow::Result<u64> {
    let input = include_str!("../../inputs/day-7.txt");
    let (_, numbers) = parse_input(input)?;

    let span = tracing::span!(Level::INFO, "try_solve");
    span.pb_set_style(&ProgressStyle::default_bar().template("{elapsed} {bar} {pos:>7}/{len:7}")?);
    span.pb_set_length(numbers.len() as u64);
    let _span = span.enter();

    let (sum, cache_accuracy) = numbers
        .iter()
        .enumerate()
        .flat_map(|(i, numbers)| {
            Span::current().pb_inc(1);
            let span = tracing::span!(Level::INFO, "", i = i);
            let _span = span.enter();
            try_solve(numbers, allowed, None, None, None)
        })
        .reduce(|acc, el| (acc.0 + el.0, acc.1.compose(el.1)))
        .unwrap_or((0, CacheStats::default()));
    tracing::info!("cache_accuracy = {:.2}", cache_accuracy.accuracy());
    Ok(sum)
}

fn part_1() -> anyhow::Result<u64> {
    solve_both(&[Operator::Add, Operator::Multiply])
}

fn part_2() -> anyhow::Result<u64> {
    solve_both(&[Operator::Add, Operator::Multiply, Operator::Concat])
}

type Memo<'a> = HashMap<&'a [Operator], u64>;

#[derive(Debug, Default, Clone)]
struct CacheStats {
    accesses: usize,
    hits: usize,
    misses: usize,
}

impl CacheStats {
    fn hit(&mut self) -> &mut Self {
        self.hits += 1;
        self.accesses += 1;
        self
    }
    fn miss(&mut self) -> &mut Self {
        self.misses += 1;
        self.accesses += 1;
        self
    }
    fn accuracy(&self) -> f64 {
        (self.hits as f64).div(self.accesses as f64)
    }
    fn compose(self, other: Self) -> Self {
        Self {
            accesses: self.accesses + other.accesses,
            hits: self.hits + other.hits,
            misses: self.misses + other.misses,
        }
    }
}

#[tailcall]
fn try_solve(
    numbers: &Numbers,
    allowed: &[Operator],
    operators: Option<&[Operator]>,
    memo: Option<Memo<'_>>,
    cache_stats: Option<&mut CacheStats>,
) -> anyhow::Result<(u64, CacheStats)> {
    let required_operators = numbers.numbers.len() - 1;
    let operators = operators.unwrap_or(&[]);
    let memo = memo.unwrap_or_default();
    let mut cs = CacheStats::default();
    let cache_stats = cache_stats.unwrap_or(&mut cs);
    tracing::trace!(?memo);
    match (allowed, operators) {
        (&[], _) => bail!("No solution"),
        (_, ops @ [tail @ .., last]) if ops.len() == required_operators => {
            let equation = Equation {
                numbers,
                operators: tail,
            };
            let res = memo
                .get(tail)
                .cloned()
                .or_else(|| {
                    tracing::trace!("cache miss");
                    let calculate = equation.calculate();
                    cache_stats.miss();
                    calculate.ok()
                })
                .context("No solution")?;
            let rhs = numbers.numbers.last().context("No numbers")?;
            let res = last.apply(res, *rhs);
            if res == numbers.result {
                Ok((res, cache_stats.clone()))
            } else {
                Err(anyhow!("No solution"))
            }
        }
        #[allow(clippy::manual_try_fold)]
        (_, ops) => allowed
            .iter()
            .fold(Err(anyhow!("No solution")), move |acc, op| {
                let new_ops = &[ops, &[*op]].concat();
                let mut memo = memo.clone();
                let equation = Equation {
                    numbers,
                    operators: ops,
                };
                let res = memo
                    .get(ops)
                    .cloned()
                    .inspect(|_| {
                        cache_stats.hit();
                    })
                    .or_else(|| {
                        tracing::trace!("cache miss");
                        let calculate = equation.calculate();
                        cache_stats.miss();
                        calculate.ok()
                    })
                    .context("No solution")?;
                let rhs = numbers.numbers[new_ops.len()];
                let res = op.apply(res, rhs);
                memo.insert(new_ops, res);
                acc.or_else(|_| {
                    try_solve(
                        numbers,
                        allowed,
                        Some(new_ops),
                        Some(memo),
                        Some(cache_stats),
                    )
                })
            }),
    }
}

fn parse_input(input: &str) -> IResult<&str, Vec<Numbers>> {
    separated_list1(tag("\n"), parse_numbers)(input)
}

fn parse_numbers(input: &str) -> IResult<&str, Numbers> {
    let number = |input| map_res(digit1, str::parse::<u64>)(input);
    map(
        separated_pair(number, tag(": "), separated_list1(tag(" "), number)),
        |(result, numbers)| Numbers { result, numbers },
    )(input)
}
