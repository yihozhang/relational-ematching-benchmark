#!/usr/bin/env python3

import argparse
import csv
from statistics import median, harmonic_mean

parser = argparse.ArgumentParser(description='Process e-matching benchmarking data')
parser.add_argument('file', type=argparse.FileType('r'))
parser.add_argument('--all-egraphs', action='store_true', help='Show data all e-graphs, not only biggest')
args = parser.parse_args()

TIMEOUT = 4e6

benches = {}
reader = csv.DictReader(args.file)
for row in list(reader):
    b = benches.setdefault(row['benchmark'], {})
    n = b.setdefault(int(row['node_size']), {})
    p = n.setdefault(row['pattern'], {})
    a = p.setdefault(row['algo'], {})

    if row['time'] == 'TO':
        time = TIMEOUT
    else:
        time = int(row['time'])

    if time == 0:
        time = 1

    a[int(row['repeat_time'])] = time

print('index,  bench,       size,  gj,  em,  total,  hmean,   best,   medn,  worst')

for bench, sizes in benches.items():
    biggest_size = max(sizes.keys())
    for size, pats in sorted(sizes.items()):
        if not (args.all_egraphs or size == biggest_size):
            continue

        for exclude_gj_index in [0, 1]:

            em_faster = 0
            em_times = []
            gj_faster = 0
            gj_times = []

            for pat, algos in pats.items():
                em = int(algos['EMatch'][0])
                gj = int(algos['GenericJoin'][exclude_gj_index])
                if gj < em:
                    gj_faster += 1
                else:
                    em_faster += 1

                em_times.append(em)
                gj_times.append(gj)

            assert len(em_times) == len(gj_times)
            total = sum(gj_times) / sum(em_times)
            fracs = [gj / em for gj, em in zip(gj_times, em_times)]
            hmean = harmonic_mean(fracs)
            print(f'{exclude_gj_index}, {bench:>10}, {size:>10}, {gj_faster:>3}, {em_faster:>3},  ' +
                  f'{total:.3f},  {hmean:.3f},  {min(fracs):.3f},  {median(fracs):.3f},  {max(fracs):.3f}')
