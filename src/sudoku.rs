use std::time::Instant;
use rand::{Rng, rng};
use varisat::{CnfFormula, ExtendFormula, Lit, Solver, dimacs};

pub fn solve_backtracking_time(matrix: &mut Vec<Vec<i8>>) -> f64 {

    let start = Instant::now();

    if solve_backtracking(matrix) {
        return start.elapsed().as_secs_f64();
    }

    f64::INFINITY
}

pub fn solve_sat_time(matrix: &mut Vec<Vec<i8>>) -> f64 {

    let start = Instant::now();

    if solve_sat(matrix) {
        return start.elapsed().as_secs_f64();
    }

    f64::INFINITY
}

pub fn get_sat_decode(matrix: &mut Vec<Vec<i8>>) -> String {

    let mut buf: Vec<u8> = Vec::new();
    dimacs::write_dimacs(&mut buf, &sudoku_to_sat(matrix)).expect("Write Dimacs err");

    String::from_utf8(buf).expect("String from utf8 err")
}

// Not using recursion for rust not guaranteeing tail call optimization. Also generally a bad idea.
pub fn solve_backtracking(matrix: &mut Vec<Vec<i8>>) -> bool {

    let size = matrix.len();

    type Cell: = (usize, usize);
    let mut positions: Vec<Cell> = Vec::new();


    for row in 0..size {
        for col in 0..size {
            if matrix[row][col] == 0 {
                positions.push((row, col));
            }
        }
    }
    
    let mut i = 0;
    while i < positions.len() {
        let pos = positions[i];
        let mut do_backtrack = true;

        for new_val in matrix[pos.0][pos.1]+1..=size as i8 {

            //println!("checking validity of {} for {}, {} (curr value {})", new_val, pos.0, pos.1, matrix[pos.0][pos.1]);

            if is_value_valid(matrix, new_val, pos){
                matrix[pos.0][pos.1] = new_val;
                i += 1;
                do_backtrack = false;
                break;
            }

        }

        if do_backtrack {
            matrix[pos.0][pos.1] = 0;
            if i == 0 {
                /*
                    This is not avoidable with a simple per-cell validity check,
                    as some puzzles can implicitly have some constraints that have no solution(s) even with valid cells.

                    Note: getting to this point can take A LOT of time and make it look like the function is looping infinitely.
                 */
                println!("No solution found.");
                return false;
            }
            i -= 1;
        }
    }

    true

}

/*
    Varisat Documentation: 
    https://jix.github.io/varisat/manual/0.2.1/lib/basic.html
*/
pub fn solve_sat(matrix: &mut Vec<Vec<i8>>) -> bool {
    let size = matrix.len();
    let formula = sudoku_to_sat(matrix);

    let mut solver = Solver::new();
    solver.add_formula(&formula);

    // Check the satisfiability of the current formula.
    if !solver.solve().unwrap() {
        return false;
    }

    let model = solver.model().unwrap();

    // Fill the grid: pick the first true n for each (r, c)
    for r in 0..size {
        for c in 0..size {
            let mut picked: i8 = 0;
            for n in 0..size {
                let lit = lit_from_indx(r, c, n, size); // 0-based var index
                if model.contains(&lit) {
                    picked = (n as i8) + 1;               // Sudoku digits are 1..=size
                    break;
                }
            }
            matrix[r][c] = picked; // stays 0 if none found (Should never happen since satisfiability was previously checked)
        }
    }
    true
}


pub fn is_value_valid(matrix: &Vec<Vec<i8>>, value: i8, pos: (usize, usize)) -> bool {

    if value == 0 {return false;}

    let size = matrix.len();
    let sub_size = size.isqrt();

    for i in 0..size { // Need to jump current pos for iterations before backtrack
        if (matrix[pos.0][i] == value && i != pos.1) || (matrix[i][pos.1] == value && i != pos.0) {return false};
    }

    let row_sub = pos.0 - (pos.0 % sub_size);
    let col_sub = pos.1 - (pos.1 % sub_size);

    for row in 0..sub_size{
        for col in 0..sub_size {
            if row + row_sub == pos.0 && col + col_sub == pos.1 {continue}

            if matrix[row + row_sub][col + col_sub] == value {return false}
        }
    }
    true
}

pub fn is_matrix_valid(matrix: &Vec<Vec<i8>>) -> Vec<(usize, usize)> {
    
    let size = matrix.len();

    type Cell: = (usize, usize);
    let mut inv_pos: Vec<Cell> = Vec::new();

    for row in 0..size {
        for col in 0..size {
            if !(is_value_valid(matrix, matrix[row][col], (row, col))) {
                inv_pos.push((row, col));
            }
        }
    }

    inv_pos
}

/*
    Note: This algorithm does not always generate actual solvable puzzles.
    It only checks essential constraints but this is not enough to guarantee it.
*/
pub fn generate_random_matrix(matrix: &mut Vec<Vec<i8>>, rnd_size: usize) {
    let size = matrix.len();

    for _ in 0..rnd_size {
        let row = rng().random_range(0..size);
        let col = rng().random_range(0..size);

        while matrix[row][col] == 0 {
            let new_value = rng().random_range(1..=size) as i8;

            if is_value_valid(matrix, new_value, (row, col)) {
                matrix[row][col] = new_value;
            }

            
        }
    }

    println!("Completed random seed.");

}

/*
    SOURCE: https://sat.inesc-id.pt/~ines/publications/aimath06.pdf
    Generates 3(n^2)
    Uses DIMACS CNF representation https://people.sc.fsu.edu/~jburkardt/data/cnf/cnf.html
*/

fn lit_from_indx(row: usize, col: usize, n: usize, size: usize) -> Lit {
    // Varisat uses 0-based var indices; `true` means positive literal.
    /*
        We need to create an index that is unique, dense and calculated in O(1) for each matrix cell regardless of its value.

        Since n has the same range of values of row and col, I decided to treat the matrix as a 3d-array (cube) with N1=N2=N3= size.

        This allows to use general array address calculation https://en.wikipedia.org/wiki/Row-_and_column-major_order
     */

    Lit::from_index(n + size * (col + size * row), true)

}

/// Build CNF for Sudoku with:
///  - ALO per cell
///  - AMO per row/col/block (for each number)
pub fn sudoku_to_sat(matrix: &Vec<Vec<i8>>) -> CnfFormula {

    let size = matrix.len();
    let sub_size = size.isqrt(); 

    let mut formula = CnfFormula::new();

    // 1) Each cell has AT LEAST ONE number
    for r in 0..size {
        for c in 0..size {
            let mut clause: Vec<Lit> = Vec::with_capacity(size);
            for n in 0..size {
                clause.push(lit_from_indx(r, c, n, size));
            }
            formula.add_clause(&clause);
        }
    }

    // 2) Each number appears at most once in each row
    for r in 0..size {
        for n in 0..size {
            for c1 in 0..size {
                for c2 in (c1 + 1)..size {
                    let a = lit_from_indx(r, c1, n, size);
                    let b = lit_from_indx(r, c2, n, size);
                    formula.add_clause(&[!a, !b]);
                }
            }
        }
    }

    // 3) Each number appears at most once in each column
    for c in 0..size {
        for n in 0..size {
            for r1 in 0..size {
                for r2 in (r1 + 1)..size {
                    let a = lit_from_indx(r1, c, n, size);
                    let b = lit_from_indx(r2, c, n, size);
                    formula.add_clause(&[!a, !b]);
                }
            }
        }
    }

    // 4) Each number appears at most once in each 3x3 sub-grid
    for br in 0..sub_size {
        for bc in 0..sub_size {
            for n in 0..size {
                // flatten block coords 0..size-1 -> (dr, dc)
                for i in 0..size {
                    for j in (i + 1)..size {
                        let r1 = br * sub_size + (i / sub_size);
                        let c1 = bc * sub_size + (i % sub_size);
                        let r2 = br * sub_size + (j / sub_size);
                        let c2 = bc * sub_size + (j % sub_size);
                        let a = lit_from_indx(r1, c1, n, size);
                        let b = lit_from_indx(r2, c2, n, size);
                        formula.add_clause(&[!a, !b]);
                    }
                }
            }
        }
    }

    // 5) Pre-filled cells clauses
    for r in 0..size {
        for c in 0..size {
            let val = matrix[r][c];
            if val != 0 {
                let n = (val - 1) as usize;
                formula.add_clause(&[lit_from_indx(r, c, n, size)]); // unit clause
            }
        }
    }

    formula
}