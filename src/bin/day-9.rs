#![feature(iter_intersperse)]

use std::{collections::BTreeMap, fmt::Display, iter::Skip, slice::Iter};

use nom::{
    character::complete::anychar,
    combinator::{map, map_opt, opt},
    multi::many0,
    sequence::pair,
    IResult,
};

fn main() -> anyhow::Result<()> {
    aoc2024::init_tracing()?;
    tracing::info!("for part 1 traces use RUST_LOG=info,day_9[part_1]=trace");
    tracing::info!("for part 2 traces use RUST_LOG=info,day_9[part_2]=trace");
    let (input, disk_usage) = parse_input()?;
    tracing::info!(part_1 = ?part_1(&input, disk_usage));
    tracing::info!(part_2 = ?part_2(&input));
    Ok(())
}

#[derive(Debug, Clone)]
struct Entry {
    id: usize,
    data: usize,
    empty: usize,
}

fn parse_entries(input: &str) -> IResult<&str, Vec<Entry>> {
    let take_digit = |input| map_opt(anychar, |c| c.to_digit(10).map(|d| d as usize))(input);
    let mut id = 0;
    let entry = map(pair(take_digit, opt(take_digit)), |(data, empty)| {
        id += 1;
        Entry {
            id: id - 1,
            data,
            empty: empty.unwrap_or(0),
        }
    });
    #[allow(clippy::let_and_return)]
    let res = many0(entry)(input);
    res
}

fn parse_input() -> anyhow::Result<(Vec<Entry>, usize)> {
    let input = include_str!("../../inputs/day-9.txt");
    let (_, entries) = parse_entries(input)?;
    let disk_usage = entries.iter().map(|entry| entry.data).sum();
    Ok((entries, disk_usage))
}

#[derive(Debug)]
struct FreeSpacePointer<'e, I: Iterator<Item = &'e Entry>> {
    entry_index: usize,
    remaining: usize,
    entries: I,
    finished: bool,
}

impl<'e> FreeSpacePointer<'e, Skip<Iter<'e, Entry>>> {
    fn new(entries: &'e [Entry]) -> Self {
        Self {
            entry_index: 0,
            remaining: entries[0].empty,
            entries: entries.iter().skip(1),
            finished: false,
        }
    }
    fn increment(&mut self) -> Option<usize> {
        self.entry_index += 1;
        match self.entries.next() {
            Some(entry) => {
                self.remaining = entry.empty;
                Some(self.remaining)
            }
            None => {
                self.finished = true;
                None
            }
        }
    }
}

fn part_1(entries: &[Entry], disk_usage: usize) -> usize {
    let _span = tracing::trace_span!("part_1").entered();
    tracing::trace!(entries = ?entries);
    let remapped: Vec<_> = entries
        .iter()
        .rev()
        .scan(FreeSpacePointer::new(entries), |state, entry| {
            let mut required_space = entry.data;
            //tracing::trace!(?state);
            if state.finished {
                return None;
            }

            let fragmented: Vec<_> = std::iter::from_fn(|| {
                if state.remaining == 0 {
                    state.increment()?;
                }
                if required_space == 0 {
                    tracing::trace!("finished fitting {}", entry.id);
                    return None;
                }
                tracing::trace!(
                    entry = ?entry,
                    state.entry_index = ?state.entry_index,
                    required_space = required_space,
                    remaining = state.remaining,
                    at = state.entry_index,
                );
                if required_space > state.remaining {
                    required_space = required_space.saturating_sub(state.remaining);
                    let res = Some((entry.id, state.entry_index, state.remaining));
                    state.increment()?;
                    res
                } else {
                    let res = Some((entry.id, state.entry_index, required_space));
                    state.remaining = state.remaining.saturating_sub(required_space);
                    required_space = 0;
                    res
                }
            })
            .collect();
            Some(fragmented.into_iter())
        })
        .flatten()
        .collect();
    let remapped = {
        let remapped = entries
            .iter()
            .map(|entry| (entry.id, entry.id, entry.data))
            .zip(remapped.iter())
            .flat_map(|(a, b)| Some(a).into_iter().chain(Some(*b)))
            .fold(BTreeMap::new(), |mut state, entry| {
                // `or_insert` required for type inference
                #[allow(clippy::unwrap_or_default)]
                state
                    .entry(entry.1)
                    .or_insert(vec![])
                    .push((entry.0, entry.2));
                state
            });
        remapped
    };
    let result_string: Vec<_> = remapped
        .iter()
        .flat_map(|(_, values)| {
            values
                .iter()
                .flat_map(|(id, data)| std::iter::repeat_n(id, *data))
        })
        .take(disk_usage)
        .collect();
    result_string
        .iter()
        .enumerate()
        .map(|(pos, id)| pos * **id)
        .sum()
}

#[derive(Debug, Clone)]
enum Block {
    Empty { size: usize },
    Data { id: usize, size: usize },
}

impl Block {
    fn size(&self) -> usize {
        match self {
            Block::Empty { size } => *size,
            Block::Data { size, .. } => *size,
        }
    }
    fn size_mut(&mut self) -> &mut usize {
        match self {
            Block::Empty { size } => size,
            Block::Data { size, .. } => size,
        }
    }
}

#[derive(Debug, Clone)]
struct Disk(Vec<Block>);

impl Disk {
    fn new(entries: &[Entry]) -> Self {
        let disk = entries
            .iter()
            .flat_map(|entry| {
                [
                    Block::Data {
                        id: entry.id,
                        size: entry.data,
                    },
                    Block::Empty { size: entry.empty },
                ]
                .into_iter()
            })
            .filter(|block| block.size() != 0)
            .collect();
        Self(disk)
    }
    fn find_block(&self, id: usize) -> Option<usize> {
        self.0.iter().position(|block| match block {
            Block::Data { id: block_id, .. } => id == *block_id,
            Block::Empty { .. } => false,
        })
    }
    fn arrange_block(mut self, block_id: usize) -> Self {
        let unchanged = self.clone();
        let arrange = || {
            let block_idx = self.find_block(block_id)?;
            let block = self.0.remove(block_idx);
            let new_position = self.0.iter().take(block_idx).position(
                |other| matches!(other, Block::Empty { size } if size >= &block.size()),
            )?;
            if let Some(prev_idx) = block_idx.checked_sub(1) {
                if let Some(prev_block) = self.0.get_mut(prev_idx) {
                    match prev_block {
                        Block::Empty { size } => {
                            *size += block.size();
                        }
                        Block::Data { .. } => {
                            let empty_space = block.size();
                            self.0
                                .insert(prev_idx + 1, Block::Empty { size: empty_space });
                        }
                    }
                }
            }
            let mut empty_block = self.0.remove(new_position);
            *empty_block.size_mut() = empty_block.size().saturating_sub(block.size());
            self.0.insert(new_position, block);
            if empty_block.size() > 0 {
                self.0.insert(new_position + 1, empty_block);
            }
            Some(self)
        };
        arrange().unwrap_or(unchanged)
    }
    fn iter(&self) -> impl Iterator<Item = Option<usize>> + '_ {
        self.0.iter().flat_map(|block| match block {
            Block::Empty { size } => std::iter::repeat(None).take(*size),
            Block::Data { id, size } => std::iter::repeat(Some(*id)).take(*size),
        })
    }
}

impl Display for Disk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self
            .iter()
            .map(|id| id.map(|id| id.to_string()).unwrap_or(".".to_string()))
            .collect::<String>();
        write!(f, "{s}")
    }
}

fn part_2(entries: &[Entry]) -> usize {
    let _span = tracing::trace_span!("part_2").entered();
    let _info_span = tracing::info_span!("part_2").entered();
    let disk = Disk::new(entries);
    tracing::trace!(?disk);
    let disk = entries
        .iter()
        .rev()
        .map(|entry| entry.id)
        .fold(disk, |disk, id| disk.arrange_block(id));
    disk.iter()
        .enumerate()
        .map(|(pos, id)| pos * id.unwrap_or(0))
        .sum()
}
