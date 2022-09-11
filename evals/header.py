import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
import numpy as np
from IPython.display import display
from pathlib import Path
import seaborn as sns
import math

#%matplotlib inline

pd.set_option("display.max_rows", 500)
pd.set_option("display.max_columns", 500)
pd.set_option("display.width", 1000)
plt.rcParams.update({"font.size": 16})


def read_benchmarks(tsv_fn, algo=None):
    df = pd.read_csv(tsv_fn, sep="\t", index_col=False)
    if "nr" in df and "cnt" in df:
        ns = df["nr"].fillna(value=df["cnt"])
        df["s_per_pair"] = df["s"] / ns
        df["s_per_bp"] = df["s"] / (ns * df["n"])
    if "e" in df:
        df["e_pct"] = 100 * df["e"]
    df["max_rss_mb"] = df["max_rss"] / 1000
    if "r" in df:
        df["r"] = df["r"].fillna(value=0)
        df = df.astype({"r": int})
    if "k" in df:
        df["k"] = df["k"].fillna(value=0)
        df = df.astype({"k": int})
    if "expanded" in df:
        df["band"] = df["expanded"] / df["|a|"]
        df = df.round({"band": 2})
    if "align" in df:
        df["align_frac"] = df["align"] / (df["precom"] + df["align"])
        df["prune_frac"] = df["prune"] / df["align"]
        df["h_approx_frac"] = df["h0"] / df["ed"]
        df["expanded_frac"] = df["expanded"] / df["explored"]
    return df


def plot_scaling(
    df,
    x,
    y,
    filename,
    trend_line="",
    xlog=False,
    ylog=False,
    split="alg",
    fit_min=None,
    cone=None,
    cone_x=10**4,
    ls="",
    x_min=0,
    title=None,
):
    fig, ax = plt.subplots(1, 1)
    fig.set_size_inches(6, 4, forward=True)

    if not isinstance(split, list):
        split = [split]

    def group_label(split_key):
        if len(split) == 1:
            return ""
        if split == ["alg", "r"]:
            alg, r = split_key
            if np.isnan(r) or r == 0:
                return ""
            if int(split_key[1]) == 1:
                return f" (exact)"
            else:
                return f" (inexact)"
        if split == ["alg", "r", "k"]:
            alg, r, k = split_key
            if np.isnan(r) or r == 0:
                return ""
            else:
                return f" ({split_key[1]}, {split_key[2]})"
        print(split)
        assert False

    groups = df.groupby(split)

    # PLOT DATA
    for split_key, group in groups:
        key = split_key[0] if isinstance(split_key, tuple) else split_key
        marker = r2marker(key, int(group["r"].max()))
        group.plot(
            x=x,
            y=y,
            ax=ax,
            alpha=1,
            zorder=3,
            rot=0,
            color=algo2color(split_key),
            marker=marker,
            ls=ls,
            legend=False,
        )

    # DRAW CONE
    # Fills the region between x**1 and x**2
    def draw_cone(x0, x_max=None):
        if x_max is None:
            x_max = 3 * df[x].max()
        # Find y0
        gb = df.groupby(["alg", x])
        index = (cone, x0)
        y0 = gb[y].get_group(index).max()
        x_range = (x0, x_max)
        y_lin = (y0, y0 * (x_max / x0) ** 1)
        y_quad = (y0, y0 * (x_max / x0) ** 2)
        ax.fill_between(x_range, y_lin, y_quad, color="grey", alpha=0.15)

    if cone is not None:
        draw_cone(x0=cone_x)

    # FIT y = x^C
    if trend_line == "poly":
        d = df.pivot(index=x, columns=split, values=[y, "r"])
        z = {}
        for algo in d[y].columns:
            s = d[y][algo].dropna()
            if fit_min:
                s = s[s.index >= fit_min(algo)]
            # s = s[s>0]
            if len(s) > 1:
                z[algo] = np.polyfit(np.log(s.index), np.log(s), 1)
        xs = list(d.index)

        # Best fit lines
        exps = {}
        for algo in z:
            key = algo[0] if isinstance(algo, tuple) else algo
            regression_line = []
            a, b = z[algo]
            plot_xs = []
            for i in xs:
                # fit only on points >= fit_min(algo)
                if fit_min and i < fit_min(algo):
                    continue
                if i > d[y][algo].dropna().index.max():
                    continue
                plot_xs.append(i)
                regression_line.append((i**a) * np.exp(b))

            label = ""
            if len(d[y][algo].dropna()) > 1:
                ax.plot(
                    plot_xs,
                    regression_line,
                    linestyle="-",
                    color=algo2color(algo),
                    alpha=0.9,
                )
                label = "$\sim n^{{{:0.2f}}}$".format(a)  ## np.exp(b)*x^a
                exps[algo] = f"{a:.2f}"
            ax.text(
                plot_xs[-1],
                regression_line[-1],
                algo2beautiful(key) + group_label(algo) + label,
                color=algo2color(algo),
                ha="right",
                va="bottom",
                size=15,
                alpha=1,
            )
        print(exps)
    else:
        for split_key, group in groups:
            max_idx = group[x].idxmax()
            label_x = group[x][max_idx]
            label_y = group[y][max_idx]
            key = split_key[0] if isinstance(split_key, tuple) else split_key
            ax.text(
                label_x,
                label_y,
                algo2beautiful(key) + group_label(split_key),
                color=algo2color(split_key),
                ha="right",
                va="bottom",
                size=15,
                alpha=1,
            )

    # ENABLE LOG SCALE
    if ylog:
        ax.set_yscale("log")
    else:
        ax.set_ylim(0)

    if xlog:
        ax.set_xscale("log")

    # SET LIMITS FOR LOG AXES
    if xlog:
        xs = df[df[x] > 0][x]
        ax.set_xlim(xs.min() / 1.5, xs.max() * 1.5)
    if ylog:
        ax.set_ylim(df[y].min() / 3, df[y].max() * 3)

    # Background
    ax.set_facecolor("#F8F8F8")

    # No border
    ax.spines["top"].set_visible(False)
    ax.spines["right"].set_visible(False)
    # Hide the left border for logarithmic x-axes.
    if xlog:
        ax.spines["left"].set_visible(False)

    # GRID: major y-axis
    ax.grid(False, axis="x", which="major")
    ax.grid(False, axis="x", which="minor")
    ax.grid(True, axis="y", which="major", color="w")
    ax.grid(False, axis="y", which="minor")

    # Ticks, no minor ticks
    ax.tick_params(
        axis="both",  # changes apply to the x-axis
        which="minor",  # both major and minor ticks are affected
        bottom=False,  # ticks along the bottom edge are off
        top=False,  # ticks along the top edge are off
        left=False,
        right=False,
        labelbottom=False,  # labels along the bottom edge are off
    )

    if xlog and xs.max() / xs.min() < 100:
        ax.tick_params(axis="x", which="minor", bottom=True)

    if x == "n":
        ax.set_xticks(
            list(
                filter(
                    lambda x: d.index.min() <= x and x <= d.index.max(),
                    [10**e for e in range(10)],
                )
            )
        )
    if x == "e_pct":
        ax.xaxis.set_major_formatter(ticker.PercentFormatter(decimals=0))
        if x_min is not None:
            ax.margins(x=0)
            ax.set_xlim(left=x_min)
    if x == "ord":
        ax.tick_params(
            axis="x",  # changes apply to the x-axis
            which="both",  # both major and minor ticks are affected
            bottom=False,  # ticks along the bottom edge are off
            top=False,  # ticks along the top edge are off
            labelbottom=False,
        )  # labels along the bottom edge are off

    # axis labels
    ax.set_xlabel(col2name(x), size=18)  # weight='bold',
    ax.set_ylabel(col2name(y), rotation=0, ha="left", size=18)
    ax.yaxis.set_label_coords(-0.10, 1.00)
  
    # Title
    if title:
        plt.title(title)

    plt.savefig(f"results/{filename}.pdf", bbox_inches="tight")


# green palette: #e1dd72, #a8c66c, #1b6535
# blue palette: #408ec6, #7a2048, #1e2761
# colorful: #cf1578, #e8d21d, #039fbe

# adobe:
# analogues blue: #DE4AFF, #625AFF, #4DC8FF
# monochromatic: #FF8746, #805C49, #CC6B37
# monochromatic violet: #F387FF, #AC68FF, #8C8CFF
# mono orange: #FFC545, FF913D, FF6047
# mono red: E8841A, FF6D29, EB2D12


def algo2color(algo):
    if isinstance(algo, tuple):
        if len(algo) == 2:
            algo, r = algo
        if len(algo) == 3:
            algo, r, k = algo
            if k == 13:
                return "#ff0000"
            if k == 15:
                return "#888800"
            if k == 20:
                return "#00ff00"
            if k == 25:
                return "#0000ff"
            if k == 30:
                return "#00ffff"
            if r == 2.0:
                return "#000000"
    palette = sns.color_palette("tab10", 10)
    d = {
        # mono red: , , EB2D12
        "dijkstra": "#5F2001",
        #'sh-noprune': '#E27121',
        #'csh-noprune': '#FF6D29',
        "sh": "#E8480C",
        "csh": "#317D32",  #'#0D7E4A',
        "edlib": "#DE4AFF",
        "wfa": "#625AFF",
        "biwfa": "#625AFF",
        # "csh+gap": "black",
        # "csh+dt": "blue",
        #'astarix-prefix': '#FF6D29',
        #'astar-prefix': '#FF6D29',
        #'astarix-prefix-illumina': '#FF6D29',
        #'astarix-seeds': '#EB2D12',
        #'astar-seeds': '#EB2D12',
        #'astarix-seeds-illumina': '#EB2D12',
        #'graphaligner': '#8C8CFF',
        #'pasgal': '#AC68FF',
        #'vargas': '#F387FF',
    }
    algo = algo.removesuffix("-noprune")
    algo = algo.removesuffix("-s_per_pair")
    algo = algo.removesuffix("-s_per_pair_no_retry")
    if algo in d:
        return d[algo]
    return "black"


def algo2beautiful(algo):
    d = {
        "dijkstra": "Dijkstra",
        "sh": "SH",
        "csh": "CSH",
        "sh-noprune": "SH (no prune)",
        "csh-noprune": "CSH (no prune)",
        "sh-s_per_pair_no_retry": "SH (excluding reordering)",
        "csh-s_per_pair_no_retry": "CSH (excluding reordering)",
        "edlib": "Edlib",
        "wfa": "WFA",
        "biwfa": "BiWFA",
    }
    algo = algo.removesuffix("-s_per_pair")
    if algo in d:
        return d[algo]
    return algo


def r2marker(algo, r):
    if algo == "dijkstra":
        return "o"
    d = {
        1: "^",
        2: "s",
    }
    if r in d:
        return d[r]
    return "o"


def col2name(col):
    d = {
        "e": "Error rate",
        "ed": "Edit distance",
        "e_pct": "Error rate",
        "expanded": "Expanded states",
        "s": "Runtime [s]",
        "n": "Sequence length [bp]",
        "s_per_pair": "Runtime per alignment [s]",
        "cpu_time": "CPU Runtime [s]",
        "max_rss": "Memory [MB]",
        "explored": "Explored states",
        "band": "Effective band",
        "ord": "Sequence pair",
    }
    if col in d:
        return d[col]
    return col
