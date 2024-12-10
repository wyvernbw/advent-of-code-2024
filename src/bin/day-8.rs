#![feature(portable_simd)]
#![feature(iter_map_windows)]
use std::{
    collections::{HashMap, HashSet},
    simd::isizex2,
};

use aoc2024::Grid;

fn main() -> anyhow::Result<()> {
    aoc2024::init_tracing()?;
    tracing::info!(part_1 = part_1()?);
    tracing::info!(part_2 = part_2()?);
    Ok(())
}

#[derive(Debug, Clone)]
enum Tile {
    Empty,
    Antenna(char),
}

fn parse_input(input: &str) -> Grid<Tile> {
    let grid = input
        .lines()
        .map(|line| {
            line.chars()
                .map(|c| match c {
                    '.' => Tile::Empty,
                    ch => Tile::Antenna(ch),
                })
                .collect()
        })
        .collect();
    Grid(grid)
}

fn map_antennas(grid: &Grid<Tile>) -> HashMap<char, Vec<(isize, isize)>> {
    grid.indexed_iter()
        .flat_map(|((i, j), tile)| match tile {
            Tile::Empty => None,
            Tile::Antenna(ch) => Some((*ch, (i as isize, j as isize))),
        })
        .fold(HashMap::new(), |mut map, (ch, pos)| {
            map.entry(ch).or_default().push(pos);
            map
        })
}

enum Resonance {
    One,
    Infinite,
}

fn part_1() -> anyhow::Result<usize> {
    solve(Resonance::One)
}

fn part_2() -> anyhow::Result<usize> {
    solve(Resonance::Infinite)
}

fn solve(resonance: Resonance) -> anyhow::Result<usize> {
    let input = include_str!("../../inputs/day-8.txt");
    let grid = parse_input(input);
    let map = map_antennas(&grid);
    let antinodes = map
        .values()
        .flat_map(|positions| {
            tracing::trace!(?positions);
            positions
                .iter()
                .cloned()
                .flat_map(|a| {
                    positions
                        .iter()
                        .cloned()
                        .filter(move |b| a != *b)
                        .map(move |b| {
                            let diff = (b.0 - a.0, b.1 - a.1);
                            Line { point: a, diff }
                        })
                })
                .flat_map(|Line { point, diff }| {
                    let p_0 = isizex2::from_array([point.0, point.1]);
                    let diff = isizex2::from_array([diff.0, diff.1]);
                    let p_1 = p_0 + diff;
                    let mut idx = match &resonance {
                        Resonance::One => 1,
                        Resonance::Infinite => 0,
                    };
                    let point_iter = std::iter::repeat_with(move || {
                        let p_0_next = p_0 - diff * isizex2::splat(idx);
                        let p_1_next = p_1 + diff * isizex2::splat(idx);
                        idx += 1;
                        (p_0_next, p_1_next)
                    });
                    match resonance {
                        Resonance::One => point_iter.take(1).collect::<Vec<_>>(),
                        Resonance::Infinite => point_iter
                            .take_while(|(p_0, p_1)| {
                                let p = [p_0, p_1];
                                grid.bounds_check((p[0][0] as usize, p[0][1] as usize))
                                    || grid.bounds_check((p[1][0] as usize, p[1][1] as usize))
                            })
                            .collect::<Vec<_>>(),
                    }
                })
        })
        .flat_map(|(p_0, p_1)| Some(p_0).into_iter().chain(Some(p_1)))
        .map(|point| {
            let i = point[0] as usize;
            let j = point[1] as usize;
            (i, j)
        })
        .filter(|&(i, j)| grid.bounds_check((i, j)))
        .inspect(|antinode| tracing::trace!(?antinode))
        .collect::<HashSet<_>>();
    for (i, row) in grid.0.iter().enumerate() {
        for (j, tile) in row.iter().enumerate() {
            match (tile, antinodes.contains(&(i, j))) {
                (Tile::Empty, false) => print!("."),
                (Tile::Empty, true) => print!("#"),
                (Tile::Antenna(ch), _) => print!("{ch}"),
            }
        }
        println!();
    }
    let total = antinodes.len();
    Ok(total)
}

struct Line {
    point: (isize, isize),
    diff: (isize, isize),
}
