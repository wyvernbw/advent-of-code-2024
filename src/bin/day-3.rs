use anyhow::bail;
use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_until},
    character::complete::digit1,
    combinator::{map, map_res, value},
    multi::many1,
    sequence::{delimited, pair, preceded, separated_pair},
    IResult,
};

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!(part_1 = ?part_1());
    tracing::info!(part_2 = ?part_2());
}

fn part_1() -> anyhow::Result<i32> {
    let input = include_str!("../../inputs/day-3.txt");
    fn parser(input: &str) -> IResult<&str, Vec<(i32, i32)>> {
        fn mul(input: &str) -> IResult<&str, (i32, i32)> {
            let number = |input| map_res(digit1, str::parse::<i32>)(input);
            let start_tag = |input| pair(take_until("mul("), tag("mul("))(input);
            alt((
                delimited(
                    start_tag,
                    separated_pair(number, tag(","), number),
                    tag(")"),
                ),
                preceded(start_tag, mul),
            ))(input)
        }
        many1(mul)(input)
    }
    let (_, muls) = parser(input)?;
    if muls.is_empty() {
        bail!("No muls found");
    }
    Ok(muls.iter().map(|(a, b)| a * b).sum())
}

#[derive(Debug, Clone)]
enum Token {
    Do,
    Dont,
    Mul(i32, i32),
}

fn part_2() -> anyhow::Result<i32> {
    let input = include_str!("../../inputs/day-3.txt");
    fn parser(input: &str) -> IResult<&str, Vec<Token>> {
        fn mul(input: &str) -> IResult<&str, Token> {
            let number = |input| map_res(digit1, str::parse::<i32>)(input);
            alt((
                map(
                    delimited(
                        tag("mul("),
                        separated_pair(number, tag(","), number),
                        tag(")"),
                    ),
                    |(a, b)| Token::Mul(a, b),
                ),
                value(Token::Do, tag("do()")),
                value(Token::Dont, tag("don't()")),
                map(pair(take(1usize), mul), |(_, t)| t),
            ))(input)
        }
        many1(mul)(input)
    }
    let (_, muls) = parser(input)?;
    if muls.is_empty() {
        bail!("No muls found");
    }
    tracing::info!(?muls);
    let (_, sum) = muls
        .iter()
        .fold((Token::Do, 0), |(state, sum), token| match (state, token) {
            (_, Token::Do) => (Token::Do, sum),
            (_, Token::Dont) => (Token::Dont, sum),
            (Token::Do, Token::Mul(a, b)) => (Token::Do, sum + a * b),
            (Token::Dont, _) => (Token::Dont, sum),
            (Token::Mul(_, _), Token::Mul(_, _)) => unreachable!(),
        });
    Ok(sum)
}
