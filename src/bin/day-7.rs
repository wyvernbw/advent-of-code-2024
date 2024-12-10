use std::{
    collections::HashMap,
    ops::Div,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

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
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use tailcall::tailcall;
use tracing::Level;
use tracing_indicatif::span_ext::IndicatifSpanExt;

fn main() -> anyhow::Result<()> {
    aoc2024::init_tracing()?;
    test()?;
    tracing::info!(part_1 = ?part_1(), "🔥");
    tracing::info!(part_2 = ?part_2(), "🔥");
    Ok(())
}

fn test() -> anyhow::Result<()> {
    let input = include_str!("../../inputs/day-7.txt");
    let (_, numbers) = parse_input(input)?;
    let wrong = numbers
        .iter()
        .filter(|numbers| {
            numbers.numbers.iter().any(|&n| n > numbers.result)
                || numbers.numbers[0] + numbers.numbers[1] > numbers.result
        })
        .count();
    let big = numbers
        .iter()
        .filter(|numbers| numbers.numbers.len() > 32)
        .count();
    tracing::info!(
        "Found {}/{} equations with more than 32 numbers",
        big,
        numbers.len()
    );
    tracing::info!(
        "Found {}/{} trivially prunable operations",
        wrong,
        numbers.len()
    );
    Ok(())
}

#[derive(Debug)]
struct Numbers {
    result: u64,
    numbers: Vec<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Operator {
    Add,
    Multiply,
    Concat,
}

impl Operator {
    fn into_key(ops: &[Operator]) -> u64 {
        ops.iter()
            .enumerate()
            .fold(ops.len() as u64, |acc, (i, op)| match op {
                Operator::Add => acc,
                Operator::Multiply => acc | (1 << (2 * i + 8)),
                Operator::Concat => acc | 1 << (2 * i + 9),
            })
    }
}

impl Operator {
    fn apply(&self, a: u64, b: u64) -> u64 {
        match self {
            Self::Add => a + b,
            Self::Multiply => a * b,
            Self::Concat => a * 10u64.pow((b as f32).log10() as u32 + 1) + b,
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
    span.pb_set_style(&ProgressStyle::default_bar().template("{elapsed} {bar:24}  {pos}/{len}")?);
    span.pb_set_length(numbers.len() as u64);
    let _span = span.enter();

    let (sum, cache_accuracy) = numbers
        .par_iter()
        .enumerate()
        .flat_map(|(i, numbers)| {
            //Span::current().pb_inc(1);
            span.pb_inc(1);
            let span = tracing::span!(Level::INFO, "", i = i);
            let _span = span.enter();
            try_solve(numbers, allowed, None, None, None)
        })
        .reduce(
            || (0, CacheStats::default().into()),
            |acc, el| (acc.0 + el.0, acc.1.compose(el.1.as_ref()).into()),
        );
    tracing::info!("cache_accuracy = {:.2}", cache_accuracy.accuracy());
    Ok(sum)
}

fn part_1() -> anyhow::Result<u64> {
    solve_both(&[Operator::Add, Operator::Multiply])
}

fn part_2() -> anyhow::Result<u64> {
    solve_both(&[Operator::Add, Operator::Multiply, Operator::Concat])
}

type Memo = HashMap<u64, u64>;

#[derive(Debug, Default)]
struct CacheStats {
    accesses: AtomicUsize,
    hits: AtomicUsize,
    misses: AtomicUsize,
}

impl CacheStats {
    fn hit(&self) -> &Self {
        self.hits.fetch_add(1, Ordering::Relaxed);
        self.accesses.fetch_add(1, Ordering::Relaxed);
        self
    }
    fn miss(&self) -> &Self {
        self.misses.fetch_add(1, Ordering::Relaxed);
        self.accesses.fetch_add(1, Ordering::Relaxed);
        self
    }
    fn accuracy(&self) -> f64 {
        (self.hits.load(Ordering::Relaxed) as f64).div(self.accesses.load(Ordering::Relaxed) as f64)
    }
    fn compose(&self, other: &Self) -> Self {
        let add_atomics = |a: &AtomicUsize, b: &AtomicUsize| {
            a.load(Ordering::Relaxed) + b.load(Ordering::Relaxed)
        };
        Self {
            hits: add_atomics(&self.hits, &other.hits).into(),
            misses: add_atomics(&self.misses, &other.misses).into(),
            accesses: add_atomics(&self.accesses, &other.accesses).into(),
        }
    }
}

#[derive(Debug)]
enum SharedMemo<'a> {
    Owned(Memo),
    MutBorrow(&'a mut Memo),
}

impl AsRef<Memo> for SharedMemo<'_> {
    fn as_ref(&self) -> &Memo {
        match self {
            Self::Owned(memo) => memo,
            Self::MutBorrow(memo) => memo,
        }
    }
}

impl AsMut<Memo> for SharedMemo<'_> {
    fn as_mut(&mut self) -> &mut Memo {
        match self {
            Self::Owned(memo) => memo,
            Self::MutBorrow(memo) => memo,
        }
    }
}

#[tailcall]
fn try_solve(
    numbers: &Numbers,
    allowed: &[Operator],
    operators: Option<&[Operator]>,
    memo: Option<SharedMemo<'_>>,
    cache_stats: Option<Arc<CacheStats>>,
) -> anyhow::Result<(u64, Arc<CacheStats>)> {
    let required_operators = numbers.numbers.len() - 1;
    let operators = operators.unwrap_or(&[]);
    let mut memo = memo.unwrap_or_else(|| SharedMemo::Owned(HashMap::new()));
    let cache_stats = cache_stats.unwrap_or_default();
    tracing::trace!(?memo);
    match (allowed, operators) {
        (&[], _) => bail!("No solution"),
        (_, ops @ [tail @ .., last]) if ops.len() == required_operators => {
            let equation = Equation {
                numbers,
                operators: tail,
            };
            let key = Operator::into_key(tail);
            let res = memo
                .as_ref()
                .get(&key)
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
                let equation = Equation {
                    numbers,
                    operators: ops,
                };
                let key = Operator::into_key(ops);
                let res = memo
                    .as_ref()
                    .get(&key)
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
                let key = Operator::into_key(new_ops);
                memo.as_mut().insert(key, res);
                if res > numbers.result {
                    return acc;
                }
                let memo = memo.as_mut();
                acc.or_else(|_| {
                    try_solve(
                        numbers,
                        allowed,
                        Some(new_ops),
                        Some(SharedMemo::MutBorrow(memo)),
                        Some(cache_stats.clone()),
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
