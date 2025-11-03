from dataclasses import dataclass


@dataclass
class Tetromino:
    box_size: int
    pixels: list[tuple[int, int]]

    def display(self) -> None:
        grid = [["." for i in range(self.box_size)] for j in range(self.box_size)]
        for x, y in self.pixels:
            grid[y][x] = "#"
        for row in grid:
            print("".join(row))

    def rotate(self, clockwise: bool) -> None:
        xo = yo = self.box_size // 2
        shift_left = False
        shift_up = False
        for i, (x, y) in enumerate(self.pixels):
            if self.box_size % 2 == 0 and x >= xo:
                x += 1
            if self.box_size % 2 == 0 and y >= yo:
                y += 1
            if clockwise:
                xr = xo + (y - yo)
                yr = yo - (x - xo)
            else:
                xr = xo - (y - yo)
                yr = yo + (x - xo)
            if self.box_size % 2 == 0 and xr >= xo:
                xr -= 1
            if self.box_size % 2 == 0 and yr >= yo:
                yr -= 1
            if xr >= self.box_size:
                shift_left = True
            if yr >= self.box_size:
                shift_up = True
            self.pixels[i] = (xr, yr)
        if shift_left:
            for i, (x, y) in enumerate(self.pixels):
                self.pixels[i] = (x - 1, y)
        if shift_up:
            for i, (x, y) in enumerate(self.pixels):
                self.pixels[i] = (x, y - 1)


def test_rotation(tetromino: Tetromino) -> None:
    tetromino.display()
    for i in range(2):
        print()
        for _ in range(4):
            print("---")
            tetromino.rotate(bool(i))
            tetromino.display()


l = Tetromino(4, [(0, 1), (1, 1), (2, 1), (3, 1)])
t = Tetromino(3, [(0, 1), (1, 1), (1, 0), (2, 1)])
o = Tetromino(2, [(0, 0), (1, 0), (1, 1), (0, 1)])
test_rotation(o)

# t = Tetromino(3, [(0, 1), (1, 1), (1, 2), (2, 1)])
