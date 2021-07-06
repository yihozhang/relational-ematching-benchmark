#!/usr/bin/env python3

import re
import numpy as np
import matplotlib.pyplot as plt
import argparse
import csv
from collections import Counter
from statistics import median, harmonic_mean, geometric_mean

parser = argparse.ArgumentParser(description='Process e-matching benchmarking data')
parser.add_argument('file', type=argparse.FileType('r'))
parser.add_argument('--all-egraphs', action='store_true', help='Show data all e-graphs, not only biggest')
args = parser.parse_args()

TIMEOUT = 60 * 1e6

patterns = {}
for row in csv.DictReader(open('patterns.csv'), skipinitialspace=True):
    # duplicate keys should be ok here, since they have equal values
    patterns[row['pattern']] = row

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

    row['time'] = time

    rpt = int(row['repeat_time'])
    a.setdefault(rpt, []).append(row)

def get_time(row):
    return row['time']

print('index,  bench,       size,  gj,  em, TO, total,  hmean,    gmean,     best,     medn,    worst')

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
                # if patterns[pat]['type'] != pattype:
                #     continue
                em_row = min(algos['EMatch'][0], key=get_time)
                gj_row = min(algos['GenericJoin'][exclude_gj_index], key=get_time)

                if em_row['result_size'] != gj_row['result_size'] and em_row['time'] != TIMEOUT:
                    print('MISMATCH!')
                    print(em_row)
                    print(gj_row)

                em = em_row['time']
                gj = gj_row['time']
                if gj < em:
                    gj_faster += 1
                else:
                    em_faster += 1

                em_times.append(em)
                gj_times.append(gj)

            assert len(em_times) == len(gj_times)
            if len(em_times) == 0:
                continue

            em_timeout = em_times.count(TIMEOUT)
            total = sum(gj_times) / sum(em_times)
            fracs = [em / gj for gj, em in zip(gj_times, em_times)]
            hmean = harmonic_mean(fracs)
            gmean = geometric_mean(fracs)
            print(f'{exclude_gj_index}, {bench:>10}, {size:>10}, {gj_faster:>3}, {em_faster:>3},  {em_timeout}, ' +
                f'{total:.3f},  {hmean:.1e},  {gmean:.1e},  {max(fracs):.1e},  {median(fracs):.1e},  {min(fracs):.1e}')


print(benches.keys())
width = 0.1

def nz(x):
    return x if x != 0 else 1

def mintime(rows):
    return min(row['time'] for row in rows)

def speedup(baseline, comp):
    print(baseline / comp)
    return np.array([
        x if x >= 1 else -1 / x
        for x in baseline / comp
    ])

def fmt_time(us):
    assert us >= 0
    if us >= 1e6:
        return '{:.0f}s'.format(us / 1e6)
    if us >= 1e3:
        return '{:.0f}ms'.format(us / 1e3)
    return '{:.0f}µs'.format(us)

def fmt_x(ratio):
    assert ratio > 0
    return '{:.1e}'.format(ratio)
    # if ratio >= 10:
    #     return '{:.0f}×'.format(ratio)
    # elif ratio >= 1:
    #     return '{:.0f}×'.format(ratio)
    # else:
        # return '1/{:.0f}×'.format(1/ratio)

def pat_rank(pat):
    n_joins = pat.count("(") - 1
    vs = Counter(re.findall(r'\?[A-Za-z0-9]+', pat))
    n_vars = sum(n for n in vs.values())
    n_var_constraints = sum(n-1 for n in vs.values())
    return (n_joins, n_var_constraints, n_vars)

# Y_TICKS = [1/100, 1/30, 1/10, 1/3, 1, 3, 10, 30, 100, 300, 1e3, 1e4, 1e5, 1e6]
Y_LABELS = '0.01 0.03 0.1 0.3 1 3 10 30 100 300 1e3 1e4 1e5 1e6'.split()
Y_TICKS  = [float(s) for s in Y_LABELS]

def plot_speedup():
    fig, axes = plt.subplots(len(benches), 1, figsize=(16, 10))
    made_legend = False
    if not isinstance(axes, np.ndarray):
        axes = [axes]
    for ax, (bench, sizes) in zip(axes, sorted(benches.items())):
        mid = (len(sizes) - 1) / 2
        assert 0 <= mid < len(sizes)

        y_ticks = Y_TICKS if bench == 'math' else Y_TICKS[:11]

        for i, (size, pats) in enumerate(sorted(sizes.items())):

            labels = list(sorted(pats.keys(), key=pat_rank))
            print(labels)

            print(i > mid)
            x = np.arange(len(labels)) + width * (i - mid) * 1.2

            gj0 = np.array([nz(mintime(pats[p]['GenericJoin'][0])) for p in labels])
            gj1 = np.array([nz(mintime(pats[p]['GenericJoin'][1])) for p in labels])
            em  = np.array([nz(mintime(pats[p]['EMatch'][0])) for p in labels])

            for n in np.log10(y_ticks):
                alpha = 1 if n == 0 else 0.1
                ax.axhline(n, alpha=alpha, zorder=-1, color='gray', linewidth=0.5)


            y0 = np.log10(em / gj0)
            y1 = np.log10(em / gj1)
            y1 = np.maximum(y1, y0) # should be as least as fast as run0

            rects0 = ax.bar(x, y0, width, color='orange', label='em / gj')
            bottom = np.maximum(0, y0)
            rects1 = ax.bar(x, y1-bottom, width, color='blue', label='em / (gj - idx)', bottom=bottom)


            for xp,p in zip(x, labels):
                if pats[p]['EMatch'][0][0]['time'] == TIMEOUT:
                    ax.text(xp, -0.3, '*',  weight='extra bold', ha='center', color='red')

            if not made_legend:
                made_legend = True
                ax.legend(loc='upper left')

        for (xp, label) in enumerate(labels):
            xp = xp - 0.3
            kwargs = dict(rotation=90, rotation_mode='anchor', fontsize=7)
            size = pats[label]['GenericJoin'][0][0]['result_size']
            t = mintime(pats[label]['GenericJoin'][0])
            ax.text(xp, -0.1, '{}, n={}'.format(fmt_time(t), size), ha='right', **kwargs)
            label = label.replace('?', '')
            ax.text(xp, 0.1, label, ha='left', **kwargs)


        x = np.arange(len(labels))
        ax.set_title(bench)
        ax.set_xticks([])
        ax.set_xticklabels([])
        ax.set_yticks(np.log10(y_ticks))
        ax.set_yticklabels(Y_LABELS[:len(y_ticks)])

plot_speedup()
plt.tight_layout()
# plt.savefig('plot.pdf')
# plt.savefig('plot.png', dpi=300)

plt.show()
