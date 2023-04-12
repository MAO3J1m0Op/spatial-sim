from pathlib import Path

path = Path(input("folder path = "))
folder_num = int(input("step count = "))
time_per_sim = float(input("time per sim = "))
prop_dumps = int(input("proportion dumps per sim = "))

path.mkdir()
input_file = (path / 'input.txt').open('x')
prop_file = (path / 'prop.csv')

alpha = [None] * 3
beta = [None] * 3
for i in range(3):
    alpha[i] = float(input(f"alpha[{i}] = "))
    beta[i] = float(input(f"beta[{i}] = "))

size = input("lattice size = ")

input_file.write(f'init {size} {alpha[0]} {alpha[1]} {alpha[2]} {beta[0]} {beta[1]} {beta[2]}\n')

for i in range(folder_num):
    folder = path / f'img{i}'
    folder.mkdir()
    for j in range(prop_dumps):
        input_file.write(f'sim {time_per_sim / prop_dumps}\n')
        input_file.write(f'dump count {prop_file}\n')
    input_file.write(f'dump img {str(folder)}\n')
    csv = path / f'img{i}.csv'
    input_file.write(f'dump csv {csv}\n')

input_file.write('exit\n')