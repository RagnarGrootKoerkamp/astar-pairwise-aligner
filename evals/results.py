#!/usr/bin/env python3

# This is a python equivalent of the evals.ipynb notebook, used as script to
# generate figures for the paper.

from header import *

# Fig 4: Tool comparison / scaling with n.
df = read_benchmarks("table/tools.tsv")
df = df[df.exit_status == 0]
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
    filename="scaling_n",
    ylog=True,
    xlog=True,
    cone="csh",
    cone_x=100,
    trend_line="poly",
    alpha=0.8,
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
    alpha=0.8,
)

# Table 1: n=10^7 slice of tool comparison, including memory.
# TABLE AT 5%, 10^7
df = read_benchmarks("table/tools.tsv")
df["alg_order"] = pd.Categorical(
    df["alg"], categories=["edlib", "biwfa", "sh", "csh"], ordered=True
)
df = df[df.exit_status == 0]
df = df[df.n == 10**7]
pt = df.pivot_table(["s_per_pair", "max_rss_mb"], ["alg_order"], ["e"])
pt = pt[["s_per_pair", "max_rss_mb"]]
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

# Band table
df = read_benchmarks("table/tools.tsv")
df = df[df.exit_status == 0]
df = df[df.n.isin([10**4, 10**5, 10**6, 10**7])]
b = df[["alg", "n", "e", "band"]].dropna()
alg_order = ["sh", "csh"]
b["alg_idx"] = b["alg"].map(lambda a: alg_order.index(a))
b = b.sort_values(by=["e", "n", "alg_idx"])
pt = pd.pivot_table(b, values="band", columns=["n"], index=["e", "alg"], sort=False)
pt.to_csv("results/table_band.csv", float_format="%.2f")
pt.style.format(precision=2).to_latex("results/table_band.tex")

# Expanded states
df = read_benchmarks("table/tools.tsv")
df = df[df.exit_status == 0]
for e in pd.unique(df.e):
    df_n = df[df.e == e]
    df_n = df_n.dropna(subset=["expanded"])
    plot_scaling(
        df_n,
        y="expanded",
        x="n",
        filename=f"expanded_e{e}",
        xlog=True,
        ylog=True,
        trend_line="poly",
    )

# Human data
def human_data(dir):
    df = read_benchmarks(f"table/human_{dir}.tsv")
    # Remove sequences for which BiWFA didn't finish in time.
    df = df[df.k.isin([0, 15])]
    df.s = df.s.clip(0, 100)
    df = df[((df.r == 0) | (df.r == 2)) & df.alg.isin(["biwfa", "edlib", "sh", "csh"])]

    # Print min/mean/max length and error rate
    df.e = df.ed / df.n
    ds = df[df.alg == "edlib"]
    # Print number of times SH is faster than both biwfa and edlib
    pt = pd.pivot_table(df, values="s", columns=["alg"], index=["id"])
    Path(f"results/stats_{dir}").write_text(
        f"""{dir}
total: {len(pt)}
SH fastest: {len(pt[(pt.sh < pt.biwfa) & (pt.sh < pt.edlib)])}
CSH fastest: {len(pt[(pt.csh < pt.biwfa) & (pt.csh < pt.edlib)])}
cnt: {len(ds)}
n: {ds.n.min()} {ds.n.mean()} {ds.n.max()}
e: {ds.e.min()} {ds.e.mean()} {ds.e.max()}
"""
        # e: {ds.e.min()} {ds.e.mean()} {ds.e.max()}
    )

    df = df[df.ed > 0]
    plot_scaling(
        df,
        y="s",
        x="ed",
        filename=f"human_{dir}",
        xlog=True,
        ylog=True,
        y_min=0.2,
        x_margin=1.1,
        # x_max = 120000,
        xticks=[10000 if dir == "chm13" else 20000, 100000],
        alpha=0.8,
        markersize=6,
        tle_tick=100,
        yticks=[1, 10, 100],
        legend=True,
    )


human_data("na12878")
human_data("chm13")

# Human data sorted plot
def human_data_sorted(dir):
    df = read_benchmarks(f"table/human_{dir}.tsv")
    df = df[df.k.isin([0, 15])]
    df = df[df.s < 100]
    df = df[((df.r == 0) | (df.r == 2)) & df.alg.isin(["biwfa", "edlib", "sh", "csh"])]
    # For each algorithm, sort datapoints by runtime.
    df["ord"] = 0

    def order_group(group):
        group = group.sort_values(by="s")
        group["ord"] = np.arange(len(group))
        return group

    df = df.groupby("alg", group_keys=False).apply(order_group)
    plot_scaling(
        df,
        y="s",
        x="ord",
        filename=f"human_sorted_{dir}",
        ylog=True,
        alpha=0.8,
        markersize=6,
        legend=True,
        tle_tick=100,
        yticks=[1, 10, 100],
        x_margin=0.01,
    )


human_data_sorted("na12878")
human_data_sorted("chm13")


# Max CSH Prune fraction
for dir in ["chm13", "na12878"]:
    r = 2
    df = read_benchmarks(f"table/human_{dir}.tsv")
    df = df[df.exit_status == 0]
    df = df[df.r == r]
    df = df[df.k == 15]
    df = df[df.alg == "csh"]
    m = (df.prune / df.t).max() * 100
    Path(f"results/prune_fraction_{dir}").write_text(f"max prune fraction: {m}")
