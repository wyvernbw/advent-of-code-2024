#![feature(iter_map_windows)]
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
use tracing::{Level, Span};
use tracing_indicatif::{span_ext::IndicatifSpanExt, IndicatifLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    let indicatif_layer = IndicatifLayer::new();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(indicatif_layer.get_stderr_writer()))
        .with(indicatif_layer)
        .init();
    tracing::info!(part_1 = ?part_1());
    tracing::info!(part_2 = ?part_2());
}

#[derive(Debug)]
struct Numbers {
    result: u64,
    numbers: Vec<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

fn part_1() -> anyhow::Result<u64> {
    let input = include_str!("../../inputs/day-7.txt");
    let (_, numbers) = parse_input(input)?;
    let sum = numbers
        .iter()
        .flat_map(|numbers| try_solve(numbers, &[Operator::Add, Operator::Multiply], None))
        .sum();
    Ok(sum)
}

fn part_2() -> anyhow::Result<u64> {
    let input = include_str!("../../inputs/day-7.txt");
    let (_, numbers) = parse_input(input)?;

    let span = tracing::span!(Level::INFO, "try_solve");
    span.pb_set_style(&ProgressStyle::default_bar().template("{elapsed} {bar} {pos:>7}/{len:7}")?);
    span.pb_set_length(numbers.len() as u64);
    let _span = span.enter();

    let sum = numbers
        .iter()
        .flat_map(|numbers| {
            Span::current().pb_inc(1);
            try_solve(
                numbers,
                &[Operator::Add, Operator::Multiply, Operator::Concat],
                None,
            )
        })
        .sum();
    Ok(sum)
}

#[tailcall]
fn try_solve(
    numbers: &Numbers,
    allowed: &[Operator],
    operators: Option<&[Operator]>,
) -> anyhow::Result<u64> {
    let required_operators = numbers.numbers.len() - 1;
    let operators = operators.unwrap_or(&[]);
    match (allowed, operators) {
        (&[], _) => bail!("No solution"),
        (_, ops) if ops.len() == required_operators => {
            let equation = Equation { numbers, operators };
            match equation.calculate() {
                res @ Ok(result) if result == numbers.result => res,
                _ => bail!("Wrong operators"),
            }
        }
        #[allow(clippy::manual_try_fold)]
        (_, ops) => allowed.iter().fold(Err(anyhow!("No solution")), |acc, op| {
            let new_ops = &[ops, &[*op]].concat();
            acc.or_else(|_| try_solve(numbers, allowed, Some(new_ops)))
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
