#![feature(hash_set_entry)]
use std::{
    collections::{HashMap, HashSet},
    convert::identity,
    hash::Hash,
};

use anyhow::Context;
use indicatif::ProgressStyle;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tailcall::tailcall;
use thiserror::Error;
use tracing::{instrument, Level, Span};
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

type Grid<T> = Vec<Vec<T>>;

fn get_input() -> Grid<char> {
    let input = include_str!("../../inputs/day-6.txt");
    input.lines().map(|line| line.chars().collect()).collect()
}

type Direction = (isize, isize);

const UP: Direction = (0, -1);
const DOWN: Direction = (0, 1);
const LEFT: Direction = (-1, 0);
const RIGHT: Direction = (1, 0);

pub trait DirectionExt {
    fn turn_right(self) -> Self;
}

impl DirectionExt for Direction {
    fn turn_right(self) -> Self {
        match self {
            UP => RIGHT,
            DOWN => LEFT,
            LEFT => UP,
            RIGHT => DOWN,
            _ => unreachable!("Direction not normalized"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    Wall,
    Guard(Direction),
}

fn parse_cell(c: char) -> Cell {
    match c {
        '.' => Cell::Empty,
        '#' => Cell::Wall,
        '^' => Cell::Guard(UP),
        'v' => Cell::Guard(DOWN),
        '>' => Cell::Guard(RIGHT),
        '<' => Cell::Guard(LEFT),
        _ => unreachable!(),
    }
}

impl From<char> for Cell {
    fn from(c: char) -> Self {
        parse_cell(c)
    }
}

fn get_grid() -> Grid<Cell> {
    let input = get_input();
    input
        .iter()
        .map(|row| row.iter().map(|c| (*c).into()).collect())
        .collect()
}

fn find_guard(grid: &Grid<Cell>) -> Option<GuardState> {
    grid.iter().enumerate().find_map(|(y, row)| {
        row.iter().enumerate().find_map(|(x, cell)| match cell {
            Cell::Guard(direction) => {
                Some(GuardState(Position(x as isize, y as isize), *direction))
            }
            _ => None,
        })
    })
}

fn part_1() -> anyhow::Result<usize> {
    let grid = get_grid();
    let guard = find_guard(&grid).context("No guard found")?;
    let visited = simulate(&grid, guard, HashSet::new())?;
    Ok(get_unique_positions(&visited).len())
}

fn find_next(
    grid: &Grid<Cell>,
    GuardState(Position(x, y), dir): GuardState,
    count: usize,
) -> Option<GuardState> {
    if count > 4 {
        return None;
    }
    let next = (x + dir.0, y + dir.1);
    let next_cell = grid
        .get(next.1 as usize)
        .and_then(|row| row.get(next.0 as usize));
    match next_cell {
        Some(Cell::Empty) | Some(Cell::Guard(_)) => Some(GuardState(Position::from(next), dir)),
        Some(Cell::Wall) => find_next(
            grid,
            GuardState(Position(x, y), dir.turn_right()),
            count + 1,
        ),
        None => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position(isize, isize);

impl From<(isize, isize)> for Position {
    fn from(pos: (isize, isize)) -> Self {
        Position(pos.0, pos.1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GuardState(Position, Direction);

#[derive(Error, Debug)]
#[error("Loop found")]
pub struct LoopError;

#[tailcall]
fn simulate(
    grid: &Grid<Cell>,
    guard_state: GuardState,
    mut visited: HashSet<GuardState>,
) -> Result<HashSet<GuardState>, LoopError> {
    let already_visited = visited.contains(&guard_state);
    visited.insert(guard_state);
    let next_guard_state = find_next(grid, guard_state, 0);
    match (already_visited, next_guard_state) {
        (true, _) => Err(LoopError),
        (false, Some(next_guard_state)) => simulate(grid, next_guard_state, visited),
        _ => Ok(visited),
    }
}

fn get_unique_positions(visited: &HashSet<GuardState>) -> HashSet<&Position> {
    visited
        .iter()
        .map(|GuardState(pos, _)| pos)
        .collect::<HashSet<_>>()
}

fn part_2() -> anyhow::Result<usize> {
    let grid = get_grid();
    let guard = find_guard(&grid).context("No guard found")?;
    let visited = simulate(&grid, guard, HashSet::new())?;
    let visited = get_unique_positions(&visited);

    let span = tracing::span!(Level::INFO, "loop check");
    tracing::info!("Checking for loops");
    span.pb_set_style(&ProgressStyle::default_bar().template("{elapsed} {bar} {pos:>7}/{len:7}")?);
    span.pb_set_length(visited.len() as u64);
    let _span = span.enter();

    let possible_obstacles = visited
        .into_iter()
        .filter(|pos| pos != &&guard.0)
        .map(|pos| {
            Span::current().pb_inc(1);
            check_loop(&grid, guard, *pos)
        })
        .filter(|has_loop| *has_loop)
        .count();
    Ok(possible_obstacles)
}

#[instrument]
fn check_loop(grid: &Grid<Cell>, guard_state: GuardState, path_pos: Position) -> bool {
    let new_grid = {
        let mut new_grid = grid.clone();
        new_grid[path_pos.1 as usize][path_pos.0 as usize] = Cell::Wall;
        new_grid
    };
    let visited = simulate(&new_grid, guard_state, HashSet::new());
    matches!(visited, Err(LoopError))
}
