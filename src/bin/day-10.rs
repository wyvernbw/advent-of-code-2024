#![feature(portable_simd)]
use std::{
    collections::HashSet,
    ops::Sub,
    simd::{isizex2, num::SimdInt},
};

use aoc2024::Grid;
use tracing::trace_span;

fn main() -> anyhow::Result<()> {
    aoc2024::init_tracing()?;
    let grid = parse_input();
    tracing::info!(part_1 = ?part_1(&grid));
    Ok(())
}

fn parse_input() -> Grid<u8> {
    let input = include_str!("../../inputs/day-10.txt");
    let grid = input
        .lines()
        .map(|line| {
            line.chars()
                .map(|c| c.to_digit(10).unwrap() as u8)
                .collect::<Vec<_>>()
        })
        .collect();
    Grid(grid)
}

fn part_1(grid: &Grid<u8>) -> usize {
    let _spans = (
        tracing::info_span!("part_1").entered(),
        // tracing::trace_span!("part_1").entered(),
    );
    let start = grid.indexed_iter().find(|(_, el)| **el == 0).unwrap().0;
    let result = find_path(grid, isizex2::from_usize(start), Path::new());
    tracing::trace!("path_tree = {:#?}", result);
    0
}

const DIR: [Position; 4] = [
    isizex2::from_array([0, 1]),
    isizex2::from_array([0, -1]),
    isizex2::from_array([1, 0]),
    isizex2::from_array([-1, 0]),
];

type Position = isizex2;
type Path = Vec<Position>;

trait ISizeX2Ext {
    fn into_indices(self) -> (usize, usize);
    fn from_usize(value: (usize, usize)) -> Self;
}

impl ISizeX2Ext for Position {
    fn into_indices(self) -> (usize, usize) {
        let [y, x] = self.to_array();
        (y as usize, x as usize)
    }
    fn from_usize((y, x): (usize, usize)) -> Self {
        Self::from_array([y as isize, x as isize])
    }
}

#[derive(Debug, Clone)]
enum PathNode {
    Node(Position, Vec<PathNode>),
    Final,
}

impl PathNode {
    fn position(&self) -> Option<Position> {
        match self {
            PathNode::Node(pos, _) => Some(*pos),
            PathNode::Final => None,
        }
    }
    fn children(&self) -> Option<&[PathNode]> {
        match self {
            PathNode::Final => None,
            PathNode::Node(_, children) => Some(children),
        }
    }
    fn has_leaf(&self, leaf: Position) -> bool {
        self.has_leaf_with(|pos| *pos == leaf)
    }
    fn has_leaf_with(&self, predicate: impl Fn(&Position) -> bool) -> bool {
        fn has_leaf_with_impl(node: &PathNode, predicate: &dyn Fn(&Position) -> bool) -> bool {
            match node {
                PathNode::Final => false,
                PathNode::Node(pos, _) if predicate(pos) => true,
                PathNode::Node(_, children) if children.is_empty() => false,
                PathNode::Node(_, children) => children
                    .iter()
                    .map(|node| node.has_leaf_with(predicate))
                    .reduce(|a, b| a || b)
                    .unwrap_or(false),
            }
        }
        has_leaf_with_impl(self, &predicate)
    }
}

fn find_path(grid: &Grid<u8>, start: Position, node: PathNode) -> PathNode {
    let current = grid[start.into_indices()];
    let options: Vec<_> = DIR
        .iter()
        .map(|dir| start + dir)
        .flat_map(|next_pos| {
            grid.get(next_pos.into_indices())
                .map(|next_cell| (next_pos, next_cell))
        })
        .filter(|(_, next_cell)| {
            next_cell
                .checked_sub(current)
                .map(|diff| diff == 1)
                .unwrap_or(false)
        })
        .collect();
    tracing::trace!(?options);
    match &options[..] {
        [] => PathNode::Final,
        _ => {
            let children: Vec<_> = options
                .into_iter()
                .flat_map(|(next_pos, _)| {
                    // only take succesful paths
                    let forward = find_path(grid, next_pos, node);
                    match forward {
                        PathNode::Node(pos, children) => {
                            let forward = children
                                .iter()
                                .filter(|node| {
                                    node.has_leaf_with(|pos| grid[pos.into_indices()] == 9)
                                })
                                .flat_map(|node| node.position().zip(node.children()))
                                .map(|(pos, path)| PathNode::Node(pos, path.to_vec()))
                                .collect();
                            Some(PathNode::Node(pos, forward))
                        }
                        PathNode::Final => None,
                    }
                })
                .collect();
            match &children[..] {
                &[] => PathNode::Final,
                _ => PathNode::Node(start, children),
            }
        }
    }
}
