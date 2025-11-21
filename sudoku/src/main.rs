use rand::seq::SliceRandom;

pub struct Sudoku {
    grid: [[u8; 9]; 9],
    entropy: [[[bool; 9]; 9]; 9],
}

impl Sudoku {
    pub fn ispossible(&self, i: usize, j: usize, num: u8) -> bool {
        for k in 0..9 {
            if self.grid[i][k] == num || self.grid[k][j] == num {
                return false;
            }
            let i0 = 3 * (i / 3);
            let j0 = 3 * (j / 3);
            for ik in 0..3 {
                for jk in 0..3 {
                    if self.grid[i0 + ik][j0 + jk] == num {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn empty() -> Self {
        Sudoku {
            grid: Default::default(),
            entropy: [[[true; 9]; 9]; 9],
        }
    }

    pub fn isfull(&self) -> bool {
        for i in 0..9 {
            for j in 0..9 {
                if self.grid[i][j] == 0 {
                    return false;
                }
            }
        }
        true
    }

    pub fn min_entropy_cell(&self) -> Option<(usize, usize)> {
        let mut res = None;
        let mut vmin = 10;
        for i in 0..9 {
            for j in 0..9 {
                if self.grid[i][j] > 0 {
                    continue;
                }
                let entro = self.entropy[i][j].iter().filter(|&x| *x).count();
                if entro < vmin {
                    vmin = entro;
                    res = Some((i, j));
                }
            }
        }
        res
    }

    pub fn remove_entry(&mut self, i: usize, j: usize) {
        let num = self.grid[i][j];
        if num == 0 {
            return;
        }
        self.grid[i][j] = 0;
        let num_indx = (num - 1) as usize;
        for k in 0..9 {
            self.entropy[i][k][num_indx] = self.ispossible(i, k, num);
            self.entropy[k][j][num_indx] = self.ispossible(k, j, num);
        }
        let i0 = 3 * (i / 3);
        let j0 = 3 * (j / 3);
        for ik in 0..3 {
            for jk in 0..3 {
                self.entropy[i0 + ik][j0 + jk][num_indx] = self.ispossible(i0 + ik, j0 + jk, num);
            }
        }
    }

    pub fn set_entry(&mut self, i: usize, j: usize, num: u8) {
        self.grid[i][j] = num;
        let num_indx = (num - 1) as usize;
        for k in 0..9 {
            self.entropy[i][k][num_indx] = false;
            self.entropy[k][j][num_indx] = false;
        }
        let i0 = 3 * (i / 3);
        let j0 = 3 * (j / 3);
        for ik in 0..3 {
            for jk in 0..3 {
                self.entropy[i0 + ik][j0 + jk][num_indx] = false;
            }
        }
    }

    pub fn display(&self) {
        for row in self.grid.iter() {
            for num in row.iter() {
                print!("{num} ");
            }
            println!();
        }
    }
}

pub fn solve_at_most_twice(sudo: &mut Sudoku, n_found: &mut u8, stop_at_first_solve: bool) {
    let (i, j) = match sudo.min_entropy_cell() {
        Some((row, col)) => (row, col),
        None => {
            *n_found += 1;
            return;
        }
    };
    let mut nums: Vec<u8> = (0..9)
        .filter(|&k| sudo.entropy[i][j][k])
        .map(|n| (n + 1) as u8)
        .collect();
    let mut rng = rand::rng();
    nums.shuffle(&mut rng);
    for num in nums.iter() {
        sudo.set_entry(i, j, *num);
        solve_at_most_twice(sudo, n_found, stop_at_first_solve);
        if *n_found > 0 && stop_at_first_solve {
            return;
        }
        if *n_found > 1 {
            return;
        }
        sudo.remove_entry(i, j);
    }
}

fn main() {
    let mut sudo = Sudoku::empty();
    let mut n_found = 0;
    solve_at_most_twice(&mut sudo, &mut n_found, true);
    println!("{n_found}");
    sudo.display();
}
