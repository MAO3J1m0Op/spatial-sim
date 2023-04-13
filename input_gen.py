from pathlib import Path
from typing import Tuple

def input_gen(
        path: Path, folder_num: int, time_per_sim: float,
        count_dumps: int, alpha: Tuple[float, float, float],
        beta: Tuple[float, float, float], lattice_size: int,
):
    path.mkdir()
    input_file = (path / 'input.txt').open('x')
    count_file = (path / 'count.csv')

    input_file.write(f'init {lattice_size} {alpha[0]} {alpha[1]} {alpha[2]} {beta[0]} {beta[1]} {beta[2]}\n')

    for i in range(folder_num):
        folder = path / f'img{i}'
        folder.mkdir()
        for j in range(count_dumps):
            input_file.write(f'sim {time_per_sim / count_dumps}\n')
            input_file.write(f'dump count {count_file}\n')
        input_file.write(f'dump img {str(folder)}\n')
        csv = path / f'img{i}.csv'
        input_file.write(f'dump csv {csv}\n')

    steps_file = path / 'steps.csv'
    input_file.write(f'dump steps {str(steps_file)}\n')
    input_file.write('exit\n')

if __name__ == '__main__':
    path = Path(input("folder path = "))
    folder_num = int(input("step count = "))
    time_per_sim = float(input("time per sim = "))
    count_dumps = int(input("count dumps per sim = "))

    alpha = [None] * 3
    beta = [None] * 3
    for i in range(3):
        alpha[i] = float(input(f"alpha[{i}] = "))
        beta[i] = float(input(f"beta[{i}] = "))
    size = input("lattice size = ")

    input_gen(path, folder_num, time_per_sim, count_dumps, tuple(alpha), tuple(beta), size)
