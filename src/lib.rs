// © Copyright 2021 Sudoku Rustic’s Authors
// Subject to the MIT License. See file LICENSE for details.

type ElementType = u8; // Capable of containg all elements plus empty, normally 0..=9.
type SizeType = usize; // Capable of indexing all cells in a grid plus one, normally 82.

/// A sudoku grid.
#[derive(Clone)]
pub struct SudokuGrid {
    /// The cells of the grid starting with top-left cell
    /// followed by rest of first row, then continues row
    /// by row.
    cells: Vec<ElementType>, // has len() = size, empty cells have value zero.

    size: SizeType,     // =elements²; normally 81.

    elements: SizeType, // =√size; values ranges from 1 to this, normally 9.
                        // Also number of cells in row/column/box.

    boxsize: SizeType,  // =√elements; normally 3.
}

impl SudokuGrid {
    /// Creates a new grid. `elements` must be a perfect
    /// square, normally 9, maximum 16 (currently).
    pub fn new(elements: ElementType) -> SudokuGrid {
        assert!(elements <= 16, "maximum number of elements is 16.");
        let boxsize = (elements as f64).sqrt() as SizeType;
        let elements = SizeType::from(elements);
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
    /// Maximum length is 256*256 = 65536
    pub fn load(cell_values: &Vec<ElementType>) -> Result<SudokuGrid, String> {
        assert!(cell_values.len() <= 65536, "Won't attempt loading grids larger than 256x256.");
        let elements_count = (cell_values.len() as f64).sqrt() as u32;
        let boxsize = (elements_count as f64).sqrt() as SizeType;
        if boxsize.pow(2).pow(2) != cell_values.len() {
            return Err(format!("Invalid input, length must be a perfect square of a perfect square. Normally 81. Was: {}", cell_values.len()));
        }
        let mut grid = SudokuGrid::new(elements_count as ElementType);
        for (i, elem) in cell_values.iter().enumerate() {
            if *elem == 0 {
                continue;
            }
            grid.cells[i] = *elem;
        }
        Ok(grid)
    }

    /// Get possible values for a cell based on its neighbors
    /// but not itself, in arbitrary order.
    fn possibilities(&self, index: SizeType) -> Vec<ElementType> {
        // `pmap` will contain `true` at `map[i]` if `i` is possible.
        let mut pmap = vec![true; self.elements + 1];
        // `pmap[0]` will not be used when constructing result, but set it to false just in case.
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

        // Now set `pmap` to false at index corresponding to element if that element is part of the
        // row, column or box already.
        // We look at each cell in the row/column/box in turn to find such elements.
        // We loop over `0..self.element` for this because that is how many cells there are in a
        // row/column/box, not because we look at each element in turn.
        // If the row/column/box contains a zero at index `i`, or looking at the `index` cell
        // itself, it will set `pmap[0]` to false, but since that has no effect when the result
        // is constructed this does not matter. Probably faster to just set `pmap[0]` than checking
        // if a write to `pmap` should be skipped.
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
        for (i, is_possible) in pmap.iter().enumerate() {
            if i > 0 && *is_possible {
                result.push(i as ElementType);
            }
        }

        result
    }
    
    /// Helper for `possibilities`. Return value in cell at `index`,
    /// except if it is `except_index` in which case it returns `0`.
    fn read_value_at_index(&self, index: SizeType, except_index: SizeType) -> usize {
        if index == except_index { return 0; }
        self.cells[index] as usize
    }
}

pub struct SudokuSolver {
    grid: SudokuGrid,

    // Index at which possibilities should be considered. If value is `>= self.grid.size` then the grid is full.
    next_index: Option<SizeType>,

    // Indexes that should be returned to when all possiblities has been exhausted at the current index.
    index_stack: Vec<SizeType>,

    // Next follows some data at every index ("this cell").

    // Points to index of next empty cell after this cell.
    index_of_next_empty: Vec<SizeType>,

    // Possible elements to try out in this cell.
    possibles: Vec<Option<Vec<ElementType>>>,
}

impl SudokuSolver {
    pub fn new(grid: SudokuGrid) -> SudokuSolver {
        let size = grid.size;
        let mut index_of_next_empty = vec![0; size];
        let mut ne = size; // Next empty cell index.
        for i in (0..size).rev() {
            // Point to the next empty cell from here.
            index_of_next_empty[i] = ne;
            if grid.cells[i] == 0 { ne = i; }
        }
        let index_stack = Vec::with_capacity(size);

        // next_index shall start at first non-empty.
        let mut next_index = None;
        if !grid.cells.is_empty() {
            match grid.cells[0] {
                // If the first cell is empty point to it.
                0 => { next_index = Some(0); }
                // But if it isn't we know the next empty one.
                _ => { next_index = Some(index_of_next_empty[0]); }
            }
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
                // Only 0x0 grids end up here.
                None
            }
            Some(x) => {
                // For rest of function x works like an index into the cells.

                // If x is past the end of the cells all the cells have
                // been filled, so we have a solution.
                if x >= self.grid.size {
                    // Return the solution and continue to
                    // other possibilities.
                    self.next_index = self.index_stack.pop();
                    return Some(self.grid.clone());
                }
                let mut x = x;
                while x < self.grid.size {
                    // This cell is empty in the original grid.

                    // If we have not visited this cell before
                    // we now need to get possible values at x.
                    let g = &self.grid;
                    let possibles_at_x =
                        self.possibles[x].get_or_insert_with({ ||
                            g.possibilities(x) });

                    match possibles_at_x.pop() {
                        Some(p) => {
                            // Try setting cell to value...
                            self.grid.cells[x] = p;
                            // ...remembering to come back here when done...
                            self.index_stack.push(x);
                            // ...but right now, check if we get anywhere
                            // with the next empty cell.
                            x = self.index_of_next_empty[x];
                        }
                        None => {
                            // We are done visiting this cell; clean up.
                            self.grid.cells[x] = 0;
                            self.possibles[x] = None;
                            // Back-track to a previous cell if any.
                            match self.index_stack.pop() {
                                None => { return None; }
                                Some(ni) => { x = ni; }
                            }
                        }
                    }
                }
                // Will only come here if x >= self.grid.size, which
                // means we could return grid as solution here, but
                // instead call self recursively once to keep the
                // success code in one place.
                self.next_index = Some(x); // Remember x when we recurse.
                self.next()
            }
        }
    }
}

/// Returns an iterator which will provide the solutions.
pub fn solutions(grid: &SudokuGrid) -> Result<SudokuSolver, String> {
    let grid = grid.clone();
    // Check grid for self-contradictions.
    for i in 0..grid.size {
        let x = grid.cells[i];
        if x == 0 {
            continue;
        }
        if x as SizeType > grid.elements {
            return Err(format!("Invalid element {}", x));
        }
        let possibles = grid.possibilities(i);
        if !possibles.contains(&x) {
            return Err("Grid contains self-contradictory cell.".to_string());
        }
    }
    Ok(SudokuSolver::new(grid))
}

/// Returns a string that is useful for output on the console.
pub fn format(grid: SudokuGrid) -> String {
    if grid.size == 0 { return "".to_string(); }
    let mut result = String::with_capacity(16*16*3);
    let minus_one_mod_row_size = grid.elements - 1;
    let minus_one_mod_boxsize = grid.boxsize - 1;
    let row_of_boxes_count = grid.boxsize * grid.elements;
    for i in 0..grid.size {
        if i > 0 && i % row_of_boxes_count == 0 {
             // Empty line before next row of boxes unless first row.
             result.push('\n');
        }
        let value = &grid.cells[i];
        result.push_str(&format_element(*value));
        // After the formatted cell value we add one of:
        if i % grid.elements == minus_one_mod_row_size {
            result.push('\n'); // after last cell on line
        } else if i % grid.boxsize == minus_one_mod_boxsize {
            result.push_str("  "); // extra space after box
        } else {
            result.push(' '); // to separate from cell after
        }
    }
    result
}

/// Parses some input as a Sudoku puzzle grid.
/// Characters '0' and '.' are interpeted as empty cells.
/// '1' to '9', 'A' to 'Z', and 'a' to 'z' as different elements.
/// Other characters are ignored.
///
/// The number of values must be a perfect square squared.
/// The maximum value must not be greater than the square
/// root of the number of values.
///
/// Typically you'd input 81 dots and numbers between 1 and 9,
/// 9 on each row.
pub fn parse(content: &str) -> Result<SudokuGrid, String> {
    // 256 is enough for a 16*16 grid.
    let mut cell_values = Vec::with_capacity(256);
    for c in content.chars() {
        let value = parse_element(c);
        if let Some(x) = value {
            cell_values.push(x);
        }
    }
    SudokuGrid::load(&cell_values)
}

/// Convert element value to string representation. 0 becomes ".",
/// 1 to 9 becomes "1" to "9", 10 to 35 becomes "A" to "Z",
/// 36 to 61 becomes "a" to "z".
fn format_element(n: ElementType) -> String {
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
fn parse_element(c: char) -> Option<ElementType> {
    match c {
        '0'..='9' => Some(c.to_digit(10).unwrap() as ElementType),
        '.' => Some(0),
        'A'..='Z' => Some(c.to_digit(36).unwrap() as ElementType),
        'a'..='z' => Some(26 + c.to_ascii_uppercase().to_digit(36).unwrap() as ElementType),
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
            "index 80: last row, last column"
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
        assert!(second_solution.is_none(), "no second solution!");
        assert!(first_solution.is_some(), "has a solution");
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
    fn given_0x0_grid_solve_shall_return_no_solutions() {
        let grid = SudokuGrid::load(&Vec::new()).unwrap();
        let solutions_vec: Vec<SudokuGrid> =
            solutions(&grid).unwrap().collect();
        assert_eq!(0, solutions_vec.len());
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
