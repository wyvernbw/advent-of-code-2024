fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!(part_1 = part_1());
    tracing::info!(part_2 = part_2());
}

fn get_input() -> Grid<char> {
    let input = include_str!("../../inputs/day-4.txt");
    input.lines().map(|line| line.chars().collect()).collect()
}

type Grid<T> = Vec<Vec<T>>;

const DIRECTIONS: [(isize, isize); 8] = [
    (-1, 0),
    (1, 0),
    (0, -1),
    (0, 1),
    (-1, -1),
    (1, -1),
    (-1, 1),
    (1, 1),
];

fn search(grid: &Grid<char>, start: (isize, isize), words: &[&str]) -> usize {
    words
        .iter()
        .map(|word| search_word(grid, start, word))
        .sum()
}

fn search_word(grid: &Grid<char>, start: (isize, isize), word: &str) -> usize {
    DIRECTIONS
        .iter()
        .map(|(dx, dy)| {
            let found = word.chars().enumerate().all(|(i, c)| {
                let letter = grid_get_at_offset(grid, start, (*dx * i as isize, *dy * i as isize));
                letter == Some(&c)
            });
            if found {
                1
            } else {
                0
            }
        })
        .sum()
}

fn part_1() -> usize {
    let input = get_input();
    input
        .iter()
        .enumerate()
        .map(|(y, row)| {
            row.iter()
                .enumerate()
                .map(|(x, _)| search(&input, (x as isize, y as isize), &["XMAS"]))
                .sum::<usize>()
        })
        .sum()
}

fn part_2() -> usize {
    let input = get_input();
    input
        .iter()
        .enumerate()
        .map(|(y, row)| {
            row.iter()
                .enumerate()
                .map(|(x, _)| is_x_mas(&input, (x as isize, y as isize)) as usize)
                .sum::<usize>()
        })
        .sum()
}

fn grid_get_at_offset(
    grid: &Grid<char>,
    (x, y): (isize, isize),
    (dx, dy): (isize, isize),
) -> Option<&char> {
    let x = x + dx;
    let y = y + dy;
    grid.get(y as usize).and_then(|row| row.get(x as usize))
}

fn is_x_mas(grid: &Grid<char>, pos: (isize, isize)) -> bool {
    let diagonal_1 = [
        grid_get_at_offset(grid, pos, (-1, -1)),
        grid_get_at_offset(grid, pos, (0, 0)),
        grid_get_at_offset(grid, pos, (1, 1)),
    ];
    let diagonal_2 = [
        grid_get_at_offset(grid, pos, (1, -1)),
        grid_get_at_offset(grid, pos, (0, 0)),
        grid_get_at_offset(grid, pos, (-1, 1)),
    ];
    let mas = [Some(&'M'), Some(&'A'), Some(&'S')];
    let mas_reversed = [Some(&'S'), Some(&'A'), Some(&'M')];
    let mas_1 = diagonal_1 == mas || diagonal_1 == mas_reversed;
    let mas_2 = diagonal_2 == mas || diagonal_2 == mas_reversed;
    mas_1 && mas_2
}
