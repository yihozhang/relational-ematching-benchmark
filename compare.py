#!/usr/bin/env python3

import re
import argparse
import csv
from collections import Counter
from statistics import median, harmonic_mean, geometric_mean

parser = argparse.ArgumentParser(description='Process e-matching benchmarking data')
parser.add_argument('file', type=argparse.FileType('r'))
parser.add_argument('--all-egraphs', action='store_true', help='Show data all e-graphs, not only biggest')
parser.add_argument('--plot', action='store_true', help='Make the plot')
args = parser.parse_args()

if args.plot:
    # only import these things if we need to
    import numpy as np
    import matplotlib.pyplot as plt

TIMEOUT = 10000 * 1e6

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
        print("TIMEOUT!!!!!")
        exit(1)
    else:
        time = int(row['time'])

    if time == 0:
        time = 1

    row['time'] = time

    rpt = int(row['repeat_time'])
    a.setdefault(rpt, []).append(row)

def get_time(row):
    return row['time']

def fmt_x(ratio):
    assert ratio > 0
    if ratio > 1e4:
        m,e = '{:.1e}'.format(ratio).split('e')
        e = int(e)
        return '{:>5}e{:d}'.format(m, e)
    elif ratio < 1:
        s = '{:>7.2f}'.format(ratio)
        return s.replace('0.', ' .', 1)
    else:
        return '{:>7.2f}'.format(ratio)
    # return '{:>6.2g}'.format(ratio)
    # m,e = '{:.1e}'.format(ratio).split('e')
    # e = int(e)
    # if e == 0:
    #     return '{:<6}'.format(m)
    # else:
    # if ratio >= 10:
    #     return '{:.0f}×'.format(ratio)
    # elif ratio >= 1:
    #     return '{:.0f}×'.format(ratio)
    # else:
        # return '1/{:.0f}×'.format(1/ratio)


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

            em_times_no_timeout = []
            gj_times_no_timeout = []

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

                if em < TIMEOUT:
                    em_times_no_timeout.append(em)
                    gj_times_no_timeout.append(gj)


            assert len(em_times) == len(gj_times)
            if len(em_times) == 0:
                continue

            em_timeout = em_times.count(TIMEOUT)
            # total = sum(em_times) / sum(gj_times)
            total = sum(em_times_no_timeout) / sum(gj_times_no_timeout)
            # fracs = [em / gj for gj, em in zip(gj_times, em_times)]
            fracs = [em / gj for gj, em in zip(gj_times_no_timeout, em_times_no_timeout)]
            hmean = harmonic_mean(fracs)
            gmean = geometric_mean(fracs)
            print(f'{exclude_gj_index}, {bench:>10}, {size:>10}, {gj_faster:>3}, {em_faster:>3},  {em_timeout}, ' +
                f'{fmt_x(total)},  {fmt_x(hmean)},  {fmt_x(gmean)},  {fmt_x(max(fracs))},  {fmt_x(median(fracs))},  {fmt_x(min(fracs))}')


# print(benches.keys())
width = 0.1

def nz(x):
    return x if x != 0 else 1

def mintime(rows):
    return min(row['time'] for row in rows)

def speedup(baseline, comp):
    # print(baseline / comp)
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

def pat_rank(pat):
    n_joins = pat.count("(") - 1
    vs = Counter(re.findall(r'\?[A-Za-z0-9]+', pat))
    n_vars = sum(n for n in vs.values())
    n_var_constraints = sum(n-1 for n in vs.values())
    return (n_joins, n_var_constraints, n_vars)

LABELS = '0.01 0.03 0.1 0.3 1 3 10 30 100 300 1e3 1e4 1e5 1e6 1e7'.split()
# LABELS = '1e-2 3e-2 1e-1 3e-1 1 3 1e1 3e1 1e2 3e2 1e3 1e4 1e5 1e6'.split()
TICKS  = [float(s) for s in LABELS]

def plot_speedup():
    fig, axes = plt.subplots(1, len(benches), figsize=(14, 16))
    made_legend = False
    if not isinstance(axes, np.ndarray):
        axes = [axes]
    for ax, (bench, sizes) in zip(axes, sorted(benches.items())):
        biggest_size = max(sizes.keys())
        mid = (len(sizes) - 1) / 2
        assert 0 <= mid < len(sizes)

        ticks = TICKS if bench == 'math' else TICKS[:11]

        for i, size in enumerate(sorted(sizes.keys())):
            pats = sizes[size]

            # labels = list(reversed(sorted(pats.keys(), key=pat_rank)))
            def size_rank(pat):
                return int(sizes[biggest_size][pat]['GenericJoin'][0][0]['result_size'])
            labels = list(sorted(pats.keys(), key=size_rank))
            # print(labels)

            x = np.arange(len(labels)) - width * (i - mid) * 1.2

            gj0 = np.array([nz(mintime(pats[p]['GenericJoin'][0])) for p in labels])
            gj1 = np.array([nz(mintime(pats[p]['GenericJoin'][1])) for p in labels])
            em  = np.array([nz(mintime(pats[p]['EMatch'][0])) for p in labels])

            for n in np.log10(ticks):
                alpha = 1 if n == 0 else 0.1
                ax.axvline(n, alpha=alpha, zorder=-1, color='gray', linewidth=0.5)


            y0 = np.log10(em / gj0)
            y1 = np.log10(em / gj1)
            y1 = np.maximum(y1, y0) # should be as least as fast as run0

            rects0 = ax.barh(x, y0, width, color='orange', label='EM / GJ')
            bottom = np.maximum(0, y0)
            rects1 = ax.barh(x, y1-bottom, width, color='blue', label='EM / (GJ - idx)', left=bottom)


            # for xp,p in zip(x, labels):
            #     if mintime(pats[p]['EMatch'][0]) == TIMEOUT:
            #         ax.text(-0.0, xp, '*',  weight='extra bold', ha='center', color='red')
            for r,p in zip(rects1, labels):
                if mintime(pats[p]['EMatch'][0]) == TIMEOUT:
                    ax.text(r.get_x() + r.get_width() + 0.1, r.get_y() - r.get_height()/2, '<',
                            ha='center', weight='extra bold', color='red')

            if not made_legend:
                made_legend = True
                ax.legend(loc='upper right')

        for (p, label) in enumerate(labels):
            p = p + 0.35
            # kwargs = dict(rotation=90, rotation_mode='anchor', fontsize=7)
            kwargs = dict(fontsize=9, va='bottom')
            size = pats[label]['GenericJoin'][0][0]['result_size']
            gjt0 = mintime(pats[label]['GenericJoin'][0])
            gjt1 = mintime(pats[label]['GenericJoin'][1])
            idxt = max(0, gjt0 - gjt1)
            emt = mintime(pats[label]['EMatch'][0])
            # ax.text(-0.1, p, '{}, n={}'.format(fmt_time(t), size), ha='right', **kwargs)
            offset = -0.1
            ax.text(-0.1, p + offset, '{} results'.format(size), ha='right', **kwargs)
            # ax.text(-0.1, p,
            # ax.text(-0.1, p + offset,
            #         '{} / ({} – {})\nn={}'.format(fmt_time(emt), fmt_time(gjt0), fmt_time(idxt), size),
            #         ha='right', **kwargs)
            # ax.text(-0.1, p,
            #         '{} / ({} – {})'.format(fmt_time(emt), fmt_time(gjt0), fmt_time(idxt)),
            #         ha='right', **kwargs)
            # ax.text(-0.1, p - 0.4, 'n={}'.format(size), ha='right', **kwargs)
            label = label.replace('?', '')
            ax.text(0.1, p + offset, label, ha='left', **kwargs)


        x = np.arange(len(labels))
        ax.set_title(bench)
        ax.set_yticks([])
        ax.set_yticklabels([])
        ax.set_xticks(np.log10(ticks))
        ax.set_xticklabels([l + '×' for l in LABELS[:len(ticks)]], rotation=-90)

if args.plot:
    plot_speedup()
    plt.tight_layout()
    plt.savefig('plot.pdf')
    plt.savefig('plot.png', dpi=300)
    plt.show()
