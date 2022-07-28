#!/usr/bin/env python3

# This is a python equivalent of the evals.ipynb notebook, used as script to
# generate figures for the paper.

from header import *

# Fig 4: Tool comparison / scaling with n.
df = read_benchmarks("table/tools.tsv")
df = df[df.exit_status == "ok"]
for e in pd.unique(df.e):
    df_n = df[df.e == e]
    plot_scaling(
        df_n,
        x="n",
        y="s_per_pair",
        filename=f"tools_e{e}",
        xlog=True,
        ylog=True,
        trend_line="poly",
        cone="csh",
        cone_x=3 * 10**4,
    )

# Fig 5: Pruning comparison
plot_scaling(
    read_benchmarks("table/scaling_n.tsv"),
    x="n",
    y="s_per_pair",
    split=["alg", "r"],
    filename="scaling_n",
    ylog=True,
    xlog=True,
    cone="csh",
    cone_x=100,
    trend_line="poly",
)

# Fig 6: Scaling with error rate
plot_scaling(
    read_benchmarks("table/scaling_e.tsv"),
    x="e_pct",
    y="s_per_pair",
    split=["alg", "r"],
    filename=f"scaling_e",
    ylog=False,
    ls="-",
)

# Table 1: n=10^7 slice of tool comparison, including memory.
# TABLE AT 5%, 10^7
df = read_benchmarks("table/tools_N1e7.tsv")
df["alg_order"] = pd.Categorical(
    df["alg"], categories=["edlib", "biwfa", "sh", "csh"], ordered=True
)
df = df[df.exit_status == "ok"]
df = df[df.n == 10**7]
pt = df.pivot_table(["s_per_pair", "max_uss"], ["alg_order"], ["e"])
pt = pt[["s_per_pair", "max_uss"]]
pt.to_csv("results/table.csv")

# Speedup
times = pt["s_per_pair"]
speedups = "Speedup at n = 10^7:\n"
for x in times:
    t = times[x]
    our_best = min(t["sh"], t["csh"]) if not np.isnan(t["sh"]) else t["csh"]
    their_best = min(t["edlib"], t["biwfa"])
    speedups += f"Speedup at {x:0.2f}: {their_best/our_best:.4}\n"
Path("results/speedup").write_text(speedups)
