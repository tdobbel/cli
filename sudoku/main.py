import itertools
import copy
import random

SudokuGrid = list[list[int]]
Entropy = list[list[list[bool]]]


def initialize_entropy() -> Entropy:
    return [[[True] * 9 for i in range(9)] for j in range(9)]


def set_number(possible: Entropy, i: int, j: int, num: int) -> Entropy:
    res = copy.deepcopy(possible)
    for k in range(9):
        res[k][j][num - 1] = False
        res[i][k][num - 1] = False
    i0 = (i // 3) * 3
    j0 = (j // 3) * 3
    for ik in range(3):
        for jk in range(3):
            res[i0 + ik][j0 + jk][num - 1] = False
    return res

def ispossible(grid: SudokuGrid, i: int, j: int, num: int) -> bool:
    for k in range(9):
        if grid[i][k] == num:
            return False
        if grid[k][j] == num:
            return False
    i0 = (i // 3) * 3
    j0 = (j // 3) * 3
    for ik in range(3):
        for jk in range(3):
            if grid[i0 + ik][j0 + jk] == num:
                return False
    return True

def remove_number(grid: SudokuGrid, possible: Entropy, i: int, j: int) -> Entropy:
    num = grid[i][j]
    grid[i][j] = 0
    new_possible = copy.deepcopy(possible)
    for k in range(9):
        new_possible[i][k][num-1] = ispossible(grid, i, k, num)
        new_possible[k][j][num-1] = ispossible(grid, k, j, num)
    i0 = (i // 3) * 3
    j0 = (j // 3) * 3
    for ik in range(3):
        for jk in range(3):
            new_possible[i0 + ik][j0 + ik][num - 1] = ispossible(grid, i0 + ik, j0 + jk, num)
    return new_possible


def isfull(grid: SudokuGrid) -> bool:
    for row in grid:
        if any(x == 0 for x in row):
            return False
    return True


def generate_random_full_grid(
    grid: SudokuGrid, possible: Entropy, i: int, j: int
) -> None:
    possible_numbers = list(k + 1 for k in range(9) if possible[i][j][k])
    if len(possible_numbers) == 0:
        return
    random.shuffle(possible_numbers)
    inext = i + j // 8
    jnext = (j + 1) % 9
    for num in possible_numbers:
        grid[i][j] = num
        if i == 8 and j == 8:
            return
        new_possible = set_number(possible, i, j, num)
        generate_random_full_grid(grid, new_possible, inext, jnext)
        if isfull(grid):
            return
    grid[i][j] = 0


def find_at_most_two_solutions(
    grid: SudokuGrid, possible: Entropy, found: list[int]
) -> None:
    vmin = 10
    imin = jmin = -1
    cntr = 0
    for i, j in itertools.product(range(9), range(9)):
        if grid[i][j] > 0:
            continue
        entropy = sum(possible[i][j])
        cntr += 1
        if entropy < vmin:
            vmin = entropy
            imin, jmin = i, j
    if cntr == 0:
        found[0] += 1
        return
    for k in range(9):
        if not possible[imin][jmin][k]:
            continue
        grid[imin][jmin] = k + 1
        new_possible = set_number(possible, imin, jmin, k + 1)
        find_at_most_two_solutions(grid, new_possible, found)
        if found[0] > 1:
            return


def unique_solution(grid: SudokuGrid, possible: Entropy) -> bool:
    found = [0]
    find_at_most_two_solutions(copy.deepcopy(grid), possible, found)
    return found[0] == 1


def remove_entries(
    grid: SudokuGrid,
    possible: Entropy,
    n_target: int,
    n_removed: int,
    indx: int,
    pairs: list[tuple[int, int]],
) -> bool:
    if n_target == n_removed:
        return True
    if indx == 81:
        return False
    i, j = pairs[indx]
    num = grid[i][j]
    new_possible = remove_number(grid, possible, i, j)
    if unique_solution(grid, new_possible) and remove_entries(grid, new_possible, n_target, n_removed + 1, indx + 1, pairs):
        return True
    grid[i][j] = num
    return remove_entries(grid, possible, n_target, n_removed, indx + 1, pairs)


def main() -> None:
    grid = [[0] * 9 for _ in range(9)]
    possible_states = [[[True] * 9 for _ in range(9)] for _ in range(9)]
    generate_random_full_grid(grid, possible_states, 0, 0)
    possible_states = [[[False] * 9 for _ in range(9)] for _ in range(9)]
    all_pairs = list(itertools.product(range(9), range(9)))
    random.shuffle(all_pairs)
    remove_entries(grid, possible_states, 50, 0, 0, all_pairs)
    for row in grid:
        print(" ".join(map(str, row)))


if __name__ == "__main__":
    main()
