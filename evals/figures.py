#!/usr/bin/env python3

# This is a python equivalent of the evals.ipynb notebook, used as script to
# generate figures for the paper.

from header import *

# Tool comparison / scaling with n.
df = read_benchmarks("table/tools_N1e7.tsv")
df = df[df.exit_status == "ok"]
df = df[df.n >= 3 * 10**3]
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

# Pruning comparison
plot_scaling(
    read_benchmarks("table/scaling_n_N1e7.tsv"),
    x="n",
    y="s_per_pair",
    split=["alg", "r"],
    filename="scaling_n",
    ylog=True,
    xlog=True,
    cone="cp-csh",
    cone_x=100,
    trend_line="poly",
)

# Scaling with error rate
plot_scaling(
    read_benchmarks("table/scaling_e_N1e6.tsv"),
    x="e_pct",
    y="s_per_pair",
    split=["alg", "r"],
    filename=f"scaling_e",
    ylog=False,
    ls="-",
)
