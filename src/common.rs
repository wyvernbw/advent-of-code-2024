use std::ops::{Index, IndexMut};

use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_tracing() -> anyhow::Result<()> {
    let env_filter = EnvFilter::from_default_env().add_directive("info".parse()?);
    let indicatif_layer = IndicatifLayer::new();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(indicatif_layer.get_stderr_writer()))
        .with(indicatif_layer)
        .with(env_filter)
        .init();
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid<T>(pub Vec<Vec<T>>);

impl<T> Grid<T> {
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter().flat_map(|row| row.iter())
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.0.iter_mut().flat_map(|row| row.iter_mut())
    }
    pub fn indexed_iter(&self) -> impl Iterator<Item = ((usize, usize), &T)> {
        self.0
            .iter()
            .enumerate()
            .flat_map(|(y, row)| row.iter().enumerate().map(move |(x, t)| ((y, x), t)))
    }
    pub fn indexed_iter_mut(&mut self) -> impl Iterator<Item = ((usize, usize), &mut T)> {
        self.0
            .iter_mut()
            .enumerate()
            .flat_map(|(y, row)| row.iter_mut().enumerate().map(move |(x, t)| ((y, x), t)))
    }
    pub fn get(&self, index: (usize, usize)) -> Option<&T> {
        self.0.get(index.0).and_then(|row| row.get(index.1))
    }
    pub fn get_mut(&mut self, index: (usize, usize)) -> Option<&mut T> {
        self.0.get_mut(index.0).and_then(|row| row.get_mut(index.1))
    }
    pub fn bounds_check(&self, index: impl TryInto<(usize, usize)>) -> bool {
        let index = index.try_into();
        index.ok().and_then(|index| self.get(index)).is_some()
    }
    pub fn width(&self) -> usize {
        self.0.len()
    }
    pub fn height(&self) -> usize {
        self.0.first().map(|row| row.len()).unwrap_or(0)
    }
}

impl<T, I> Index<I> for Grid<T>
where
    I: Into<(usize, usize)>,
{
    type Output = T;

    fn index(&self, index: I) -> &Self::Output {
        let (y, x): (usize, usize) = index.into();
        &self.0[y][x]
    }
}

impl<T, I> IndexMut<I> for Grid<T>
where
    I: Into<(usize, usize)>,
{
    fn index_mut(&mut self, index: I) -> &mut T {
        let (y, x): (usize, usize) = index.into();
        &mut self.0[y][x]
    }
}
