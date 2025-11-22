import itertools
import copy
import random


class Sudoku:
    def __init__(self) -> None:
        self.grid = [[0] * 9 for _ in range(9)]
        self.possible = [[[True] * 9 for _ in range(9)] for _ in range(9)]

    def set_number(self, i: int, j: int, num: int) -> None:
        self.grid[i][j] = num
        for k in range(9):
            self.possible[k][j][num - 1] = False
            self.possible[i][k][num - 1] = False
        i0 = (i // 3) * 3
        j0 = (j // 3) * 3
        for ik in range(3):
            for jk in range(3):
                self.possible[i0 + ik][j0 + jk][num - 1] = False

    def ispossible(self, i: int, j: int, num: int) -> bool:
        for k in range(9):
            if self.grid[i][k] == num or self.grid[k][j] == num:
                return False
        i0 = (i // 3) * 3
        j0 = (j // 3) * 3
        for ik, jk in itertools.product(range(3), range(3)):
            if self.grid[i0 + ik][j0 + jk] == num:
                return False
        return True

    def remove_number(self, i: int, j: int) -> int:
        num = self.grid[i][j]
        if num == 0:
            return num
        self.grid[i][j] = 0
        for k in range(9):
            self.possible[i][k][num - 1] = self.ispossible(i, k, num)
            self.possible[k][j][num - 1] = self.ispossible(k, j, num)
        i0 = (i // 3) * 3
        j0 = (j // 3) * 3
        for ik in range(3):
            for jk in range(3):
                self.possible[i0 + ik][j0 + ik][num - 1] = self.ispossible(
                    i0 + ik, j0 + jk, num
                )
        return num

    def get_min_entropy_cell(self) -> tuple[int, int]:
        vmin = 10
        imin = jmin = -1
        for i, j in itertools.product(range(9), range(9)):
            if self.grid[i][j] > 0:
                continue
            entropy = sum(self.possible[i][j])
            if entropy < vmin:
                vmin = entropy
                imin, jmin = i, j
        return imin, jmin

    def display(self) -> None:
        for row in self.grid:
            print(" ".join(map(str, row)))


def solve_at_most_twice(
    sudo: Sudoku, n_found: list[int], stop_fast: bool = False
) -> None:
    i, j = sudo.get_min_entropy_cell()
    if i == -1:
        n_found[0] += 1
        return
    nums = list(k + 1 for k in range(9) if sudo.possible[i][j][k])
    random.shuffle(nums)
    for num in nums:
        sudo.set_number(i, j, num)
        solve_at_most_twice(sudo, n_found, stop_fast)
        if n_found[0] > 0 and stop_fast:
            return
        if n_found[0] > 1:
            return
        sudo.remove_number(i, j)


def remove_entries(
    sudo: Sudoku, n_target: int, n_removed: int, pairs: list[tuple[int, int]]
) -> bool:
    if n_removed == n_target:
        return True
    if len(pairs) == 0:
        return False
    i, j = pairs[0]
    num = sudo.remove_number(i, j)
    new_sudo = copy.deepcopy(sudo)
    n_found = [0]
    solve_at_most_twice(new_sudo, n_found)
    if n_found[0] == 1 and remove_entries(sudo, n_target, n_removed + 1, pairs[1:]):
        return True
    sudo.set_number(i, j, num)
    return remove_entries(sudo, n_target, n_removed, pairs[1:])


def generate_random_grid(n_clues: int) -> Sudoku:
    sudo = Sudoku()
    solve_at_most_twice(sudo, [0], True)
    pairs = list(itertools.product(range(9), range(9)))
    random.shuffle(pairs)
    remove_entries(sudo, 81 - n_clues, 0, pairs)
    return sudo


def main() -> None:
    sudo = generate_random_grid(25)
    sudo.display()


if __name__ == "__main__":
    main()
