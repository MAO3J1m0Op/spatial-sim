from pathlib import Path

from input_gen import input_gen

# Constant parameters
folder_num = 40
sim_time = 0.25
count_dumps = 100
size = 45
alpha1 = -0.1
alpha2 = -0.5
beta1 = 0.6
beta2 = 0.2

params = (-0.4, -0.2, -0.1, 0.0, 0.1, 0.2, 0.4)

for alpha3 in params:
    for beta3 in params:
        input_gen(
            Path(f'data/{alpha3}.{beta3}'),
            folder_num, sim_time, count_dumps,
            (alpha1, alpha2, alpha3),
            (beta1, beta2, beta3), size) 
