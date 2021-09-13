// Terminology: https://en.wikipedia.org/wiki/Glossary_of_Sudoku
// Inspiration: https://youtu.be/G_UYXzGuqvM

/// A sudoku grid.
#[derive(Clone)]
pub struct SudokuGrid {
    /// The cells of the grid starting with top-left cell
    /// followed by rest of first row, then continues row
    /// by row.
    cells: Vec<usize>, // has len() = size, empty cells have value zero.
    size: usize,     // =elements²; normally 81.
    elements: usize, // =√size; values ranges from 1 to this, normally 9.
    boxsize: usize,  // =√elements; normally 3.
}

impl SudokuGrid {
    /// Creates a new grid. `elements` must be a perfect
    /// square, normally 9, maximum 16 (currently).
    pub fn new(elements: usize) -> SudokuGrid {
        assert!(elements <= 16, "maximum number of elements is 16.");
        let boxsize = (elements as f64).sqrt() as usize;
        assert_eq!(
            elements,
            boxsize.pow(2),
            "elements must be a perfect square."
        );
        let size = elements.pow(2);
        let cells = vec![0; size];
        SudokuGrid {
            cells,
            size,
            elements,
            boxsize,
        }
    }

    /// Returns a grid preloaded with the values in `cell_values`.
    /// Length of argument must be a perfect square of a perfect square.
    /// `cell_values` represents the cells of the grid starting with
    /// top-left cell followed by rest of first row, then continues row
    /// by row.
    ///
    /// A value of 0 means empty.
    /// Any other number is an element in that cell.
    pub fn load(cell_values: &Vec<usize>) -> Result<SudokuGrid, String> {
        let elements = (cell_values.len() as f64).sqrt() as usize;
        let boxsize = (elements as f64).sqrt() as usize;
        if boxsize.pow(2).pow(2) != cell_values.len() {
            return Err(format!("Invalid input, length must be a perfect square of a perfect square. Normally 81. Was: {}", cell_values.len()));
        }
        let mut grid = SudokuGrid::new(elements);
        for (i, elem) in cell_values.iter().enumerate() {
            if *elem == 0 {
                continue;
            }
            grid.cells[i] = *elem;
        }
        Ok(grid)
    }

    /// Like `possibilities` except the returned vec is suitable
    /// for popping off smallest values first.
    fn possibilities_reverse(&mut self, index: usize) -> Vec<usize> {
        let mut p = self.possibilities(index);
        p.reverse();
        p
    }

    /// Get possible values for a cell based on its neighbors
    /// but not itself, in ascending order.
    fn possibilities(&self, index: usize) -> Vec<usize> {
        // `pmap` will contain `true` at `map[i]` if `i` is possible.
        let mut pmap = vec![true; self.elements + 1];
        pmap[0] = false;
        let rowstart_index = (index / self.elements) * self.elements;
        let colstart_index = index % self.elements;

        // Which box column, has value in range `0..self.boxsize`.
        let boxcol = (index - rowstart_index) / self.boxsize;
        // Which box row, has value in range `0..self.boxsize`.
        let boxrow = (index - colstart_index) / (self.elements * self.boxsize);
        // Top left corner of box:
        let boxbase_index = boxrow * self.boxsize * self.elements // row
                            + boxcol * self.boxsize; // column
        for i in 0..self.elements {
            // row
            pmap[self.read_value_at_index(i + rowstart_index, index)] = false;
            // column
            pmap[self.read_value_at_index(
                (i * self.elements) + colstart_index, index)] = false;
            // box
            pmap[self.read_value_at_index(
                // This calculation is dense?
                // Could make two for loops of 0..self.boxsize instead
                boxbase_index
                 + (i % self.boxsize) // loop columns
                 + (i / self.boxsize) * self.elements // loop rows
                , index)] = false;
        }

        // Construct result.
        let mut result = Vec::with_capacity(self.elements);
        for i in 1..=self.elements {
            if pmap[i] {
                result.push(i);
            }
        }

        result
    }
    
    /// Helper for `possibilities`. Return value in cell at `index`,
    /// except if it is `except_index` in which case it returns `0`.
    fn read_value_at_index(&self, index: usize, except_index: usize) -> usize {
        if index == except_index { return 0; }
        return self.cells[index];
    }
}

pub struct SudokuSolver {
    grid: SudokuGrid,

    next_index: Option<usize>,
    index_stack: Vec<usize>,

    // some data at every index
    index_of_next_empty: Vec<usize>,
    possibles: Vec<Option<Vec<usize>>>,
}

impl SudokuSolver {
    pub fn new(grid: SudokuGrid) -> SudokuSolver {
        let size = grid.size;
        let mut index_of_next_empty = vec![0; size];
        let mut ne = size;
        for i in (0..size).rev() {
            index_of_next_empty[i] = ne;
            if grid.cells[i] == 0 { ne = i; }
        }
        let index_stack = Vec::with_capacity(size);
        // next_index shall start at first non-empty.
        let mut next_index = Some(0);
        if grid.cells.len() > 0 && grid.cells[0] != 0 {
            next_index = Some(index_of_next_empty[0]);
        }

        SudokuSolver {
            grid,
            next_index,
            index_stack,
            index_of_next_empty,
            possibles: vec![None; size],
        }
    }
}

impl Iterator for SudokuSolver {
    type Item = SudokuGrid;
    fn next(&mut self) -> Option<Self::Item> {
        match self.next_index {
            None => {
                return None;
            }
            Some(x) => {
                if x >= self.grid.size {
                    // We have a solution, return it and continue
                    // other possibilities.
                    self.next_index = self.index_stack.pop();
                    return Some(self.grid.clone());
                }
                let mut x = x;
                while x < self.grid.size {
                    let g = &mut self.grid;
                    let possibles_at_x =
                        self.possibles[x].get_or_insert_with({ ||
                            g.possibilities_reverse(x) });
                    match possibles_at_x.pop() {
                        None => {
                            self.grid.cells[x] = 0;
                            self.possibles[x] = None; // might need to recalc
                            match self.index_stack.pop() {
                                None => { return None; }
                                Some(ni) => { x = ni; }
                            }
                        }
                        Some(p) => {
                            self.grid.cells[x] = p;
                            self.index_stack.push(x);
                            x = self.index_of_next_empty[x];
                        }
                    }
                }
                self.next_index = Some(x);
                return self.next();
            }
        }
    }
}

pub fn solutions(grid: &SudokuGrid) -> Result<SudokuSolver, String> {
    let grid = grid.clone();
    // Check grid self-validity.
    for i in 0..grid.size {
        let x = grid.cells[i];
        if x == 0 {
            continue;
        }
        if x > grid.elements {
            return Err(format!("Invalid element {}", format_element(x)));
        }
        let possibles = grid.possibilities(i);
        if !possibles.contains(&x) {
            return Err("Grid contains self-contradictory cell.".to_string());
        }
    }
    Ok(SudokuSolver::new(grid))
}

pub fn format(grid: SudokuGrid) -> String {
    if grid.size == 0 { return "".to_string(); }
    let mut result = String::new();
    let minus_one_mod_row_size = grid.elements - 1;
    let minus_one_mod_boxsize = grid.boxsize - 1;
    let row_of_boxes_count = grid.boxsize * grid.elements;
    for i in 0..grid.size {
        if i > 0 && i % row_of_boxes_count == 0 {
            result.push_str("\n"); // empty line before next row of boxes
        }
        let value = *(&grid.cells[i]);
        result.push_str(&format_element(value));
        if i % grid.elements == minus_one_mod_row_size {
            result.push_str("\n"); // after last cell on line
        } else if i % grid.boxsize == minus_one_mod_boxsize {
            result.push_str("  "); // extra space after box
        } else {
            result.push_str(" "); // to separate from cell after
        }
    }
    result
}

pub fn parse(content: &String) -> Result<SudokuGrid, String> {
    // 256 is enough for a 16*16 grid.
    let mut cell_values = Vec::with_capacity(256);
    for c in content.chars() {
        let value = parse_element(c);
        match value {
            Some(x) => {
                cell_values.push(x);
            }
            _ => { /* formatting */ }
        }
    }
    return SudokuGrid::load(&cell_values);
}

/// Convert element value to string representation. 0 becomes ".",
/// 1 to 9 becomes "1" to "9", 10 to 35 becomes "A" to "Z",
/// 36 to 61 becomes "a" to "z".
fn format_element(n: usize) -> String {
    let n = n as u32;
    match n {
        0 => ".".to_string(),
        1..=9 => n.to_string(),
        10..=35 => char::from_digit(n, 36)
            .unwrap()
            .to_ascii_uppercase()
            .to_string(),
        36..=61 => char::from_digit(n - 26, 36)
            .unwrap()
            .to_ascii_lowercase()
            .to_string(),
        _ => {
            panic!("don't know how to format that")
        }
    }
}

/// Converts from char to element value. '.' becomes 0,
/// '0' to '9' becomes 0 to 9, 'A' to 'Z' becomes 10 to 35,
/// 36 to 61. Other chars become nothing.
fn parse_element(c: char) -> Option<usize> {
    match c {
        '0'..='9' => Some(c.to_digit(10).unwrap() as usize),
        '.' => Some(0),
        'A'..='Z' => Some(c.to_digit(36).unwrap() as usize),
        'a'..='z' => Some(26 + c.to_ascii_uppercase().to_digit(36).unwrap() as usize),
        _ => None,
    }
}

#[cfg(test)]
mod parse_format {
    use super::*;

    #[test]
    fn given_nonsense_parse_element_should_return_none() {
        assert_eq!(parse_element('/'), None);
        assert_eq!(parse_element('\n'), None);
        assert_eq!(parse_element('-'), None);
        assert_eq!(parse_element('|'), None);
        assert_eq!(parse_element(';'), None);
        assert_eq!(parse_element('*'), None);
        assert_eq!(parse_element(' '), None);
        assert_eq!(parse_element('Å'), None);
    }

    #[test]
    fn given_0_format_element_should_return_dot() {
        assert_eq!(format_element(0), ".");
    }

    #[test]
    fn given_dot_parse_element_should_return_0() {
        assert_eq!(parse_element('.'), Some(0));
    }

    #[test]
    fn given_0_parse_element_should_return_0() {
        assert_eq!(parse_element('0'), Some(0));
    }

    #[test]
    fn given_1_format_element_should_return_string_1() {
        assert_eq!(format_element(1), "1");
    }

    #[test]
    fn given_1_parse_element_should_return_1() {
        assert_eq!(parse_element('1'), Some(1));
    }

    #[test]
    fn given_9_format_element_should_return_string_9() {
        assert_eq!(format_element(9), "9");
    }

    #[test]
    fn given_9_parse_element_should_return_9() {
        assert_eq!(parse_element('9'), Some(9));
    }

    #[test]
    fn given_10_format_element_should_return_big_a() {
        assert_eq!(format_element(10), "A");
    }

    #[test]
    fn given_big_a_parse_element_should_return_10() {
        assert_eq!(parse_element('A'), Some(10));
    }

    #[test]
    fn given_16_format_element_should_return_big_g() {
        assert_eq!(format_element(16), "G");
    }

    #[test]
    fn given_big_g_parse_element_should_return_16() {
        assert_eq!(parse_element('G'), Some(16));
    }

    #[test]
    fn given_35_format_element_should_return_big_z() {
        assert_eq!(format_element(35), "Z");
    }

    #[test]
    fn given_big_z_parse_element_should_return_35() {
        assert_eq!(parse_element('Z'), Some(35));
    }

    #[test]
    fn given_36_format_element_should_return_small_a() {
        assert_eq!(format_element(36), "a");
    }

    #[test]
    fn given_small_a_parse_element_should_return_36() {
        assert_eq!(parse_element('a'), Some(36));
    }

    #[test]
    fn given_61_format_element_should_return_small_z() {
        assert_eq!(format_element(61), "z");
    }

    #[test]
    fn given_small_z_parse_element_should_return_61() {
        assert_eq!(parse_element('z'), Some(61));
    }

    #[test]
    fn given_0x0_grid_format_shall_return_empty_string() {
        let grid = SudokuGrid::load(&Vec::new()).unwrap();
        let result = format(grid);
        assert_eq!("", result);
    }

    #[test]
    fn given_1x1_grid_format_shall_return_1_as_string() {
        let grid = SudokuGrid::load(&vec![1]).unwrap();
        let result = format(grid);
        assert_eq!("1\n", result);
    }
}

#[cfg(test)]
mod solving {
    use super::*;

    #[test]
    fn given_4x4_grid_possibilities_returns_correct_values() {
        let input = vec![1, 2, 0, 0, //
                         3, 0, 0, 1, //
                         2, 0, 0, 4, //
                         4, 0, 0, 0];
        let grid = SudokuGrid::load(&input).unwrap();
        assert_eq!(grid.possibilities(0), vec![1], "index 0");
        assert_eq!(grid.possibilities(1), vec![2, 4], "index 1");
        assert_eq!(grid.possibilities(2), vec![3, 4], "index 2");
        assert_eq!(grid.possibilities(3), vec![3], "index 3");

        assert_eq!(grid.possibilities(4), vec![3], "index 4");
        assert_eq!(grid.possibilities(5), vec![4], "index 5");
        assert_eq!(grid.possibilities(6), vec![2, 4], "index 6");
        assert_eq!(grid.possibilities(7), vec![1, 2], "index 7");

        assert_eq!(grid.possibilities(8), vec![2], "index 8");
        assert_eq!(grid.possibilities(9), vec![1, 3], "index 9");
        assert_eq!(grid.possibilities(10), vec![1, 3], "index 10");
        assert_eq!(grid.possibilities(11), vec![3, 4], "index 11");

        assert_eq!(grid.possibilities(12), vec![4], "index 12");
        assert_eq!(grid.possibilities(13), vec![1, 3], "index 13");
        assert_eq!(grid.possibilities(14), vec![1, 2, 3], "index 14");
        assert_eq!(grid.possibilities(15), vec![2, 3], "index 15");
    }

    #[test]
    fn given_9x9_grid_posibilites_returns_correct_for_selected() {
        let input = "
            7.9 4.2 8.3
            .5. ... .2.
            ... 653 ...

            1.. 5.7 ..8
            ..7 ... 6..
            89. 1.6 .47

            ..1 .7. 4..
            ..5 ... 7..
            ..4 .8. 3..
        "
        .to_string();
        let grid = parse(&input).unwrap();
        assert_eq!(
            grid.possibilities(4),
            vec![1],
            "index 4: top row, middlest column"
        );
        assert_eq!(
            grid.possibilities(26),
            vec![1, 4, 9],
            "index 26: third row, last column"
        );
        assert_eq!(
            grid.possibilities(80),
            vec![1, 2, 5, 6, 9],
            "index 26: last row, last column"
        );
    }

    #[test]
    fn given_9x9_grid_solutions_is_one_solution_which_is_correct() {
        let input = "
            7.9 4.2 8.3
            .5. ... .2.
            ... 653 ...

            1.. 5.7 ..8
            ..7 ... 6..
            89. 1.6 .47

            ..1 .7. 4..
            ..5 ... 7..
            ..4 .8. 3..
        "
        .to_string();
        let answer_key_input = "
            769 412 853
            453 798 126
            218 653 974

            136 547 298
            547 829 631
            892 136 547

            621 375 489
            385 964 712
            974 281 365
        "
        .to_string();
        let grid = parse(&input).unwrap();
        let answer_key = parse(&answer_key_input).unwrap();
        let mut solution_iterator = solutions(&grid).unwrap();
        let first_solution = solution_iterator.next();
        let second_solution = solution_iterator.next();
        assert!(second_solution.is_none());
        assert!(first_solution.is_some());
        assert_eq!(answer_key.cells, first_solution.unwrap().cells);
    }

    #[test]
    fn given_contradictory_4x4_grid_should_get_no_solution_iterator() {
        let input = "
        1234
        4321
        .2..
        ....
        "
        .to_string();
        let grid = parse(&input).unwrap();
        let solution_iterator = solutions(&grid);
        assert!(solution_iterator.is_err());
    }

    #[test]
    fn given_already_solved_4x4_grid_should_return_it() {
        let input = "
        1234
        4321
        3142
        2413
        "
        .to_string();
        let grid = parse(&input).unwrap();

        let mut solution_iterator = solutions(&grid).unwrap();
        let first_solution = solution_iterator.next();
        let second_solution = solution_iterator.next();
        assert!(second_solution.is_none());
        assert!(first_solution.is_some());
        assert_eq!(grid.cells, first_solution.unwrap().cells);
    }

    #[test]
    fn given_particular_4x4_grid_should_return_three_solutions() {
        let input = "
        12..
        43..
        ....
        ...1
        "
        .to_string();
        let grid = parse(&input).unwrap();

        let solutions_vec: Vec<SudokuGrid> = solutions(&grid).unwrap().collect();
        assert_eq!(3, solutions_vec.len());
    }

    #[test]
    fn given_particular_4x4_grid_should_return_just_one_correct_solution() {
        let input = "
        .234
        43..
        ....
        ...1
        "
        .to_string();
        let answer_key_input = "
        1234
        4312
        2143
        3421
        ".to_string();
        let grid = parse(&input).unwrap();
        let answer_key = parse(&answer_key_input).unwrap();
        let mut solutions_vec: Vec<SudokuGrid> =
            solutions(&grid).unwrap().collect();
        assert_eq!(1, solutions_vec.len());
        let the_solution = solutions_vec.pop().unwrap();
        assert_eq!(answer_key.cells, the_solution.cells);
    }

    #[test]
    fn given_0x0_grid_solve_shall_return_0x0_grid() {
        let grid = SudokuGrid::load(&Vec::new()).unwrap();
        let mut solutions_vec: Vec<SudokuGrid> =
            solutions(&grid).unwrap().collect();
        assert_eq!(1, solutions_vec.len());
        let the_solution = solutions_vec.pop().unwrap();
        assert!(the_solution.cells.is_empty());
    }

    #[test]
    fn given_1x1_grid_solve_shall_return_1x1_grid_with_1_in_cell() {
        let grid = SudokuGrid::load(&vec![0]).unwrap();
        let mut solutions_vec: Vec<SudokuGrid> =
            solutions(&grid).unwrap().collect();
        assert_eq!(1, solutions_vec.len());
        let the_solution = solutions_vec.pop().unwrap();
        assert_eq!(vec![1], the_solution.cells);
    }
}
