# Instructions for compiling:
# - A* Pairwise Aligner:
#   - cargo build --release
# - edlib:
#   - From this directory:
#   - cd ..
#   - Clone the fork, which contains an updated binary to handle the WFA input format.
#   - git clone https://github.com/RagnarGrootKoerkamp/edlib
#   - cd edlib/build
#   - cmake -D CMAKE_BUILD_TYPE=Release .. && make
# - WFA:
#   - From this directory:
#   - cd ..
#   - git clone https://github.com/smarco/WFA wfa
#   - cd wfa
#   - make clean all
#
# Instructions for running:
# - snakemake -p -c all run_benchmark

pa_bin    = "target/release/pairwise-aligner"
edlib_bin = "../edlib/build/bin/edlib-aligner"
wfa_bin   = "../wfa/bin/align_benchmark"

TIMELIMIT       = "(timeout 1s"
TIMELIMITEND    = ") || true"

DIJKSTRA_CMD    = '{TIMELIMIT} {pa_bin} -i {input} -o data/runs/{wildcards.cnt}x-n{wildcards.n}-e{wildcards.e}.dijkstra.bench.band -a Dijkstra --silent {TIMELIMITEND}'
PA_CMD          = '{TIMELIMIT} {pa_bin} -i {input} -o data/runs/{wildcards.cnt}x-n{wildcards.n}-e{wildcards.e}.pa.bench.band -l {params.l} --silent {TIMELIMITEND} || true'
PA_NO_PRUNE_CMD = '{TIMELIMIT} {pa_bin} -i {input} -o data/runs/{wildcards.cnt}x-n{wildcards.n}-e{wildcards.e}.pa-no-prune.bench.band -l {params.l} --no-prune --silent {TIMELIMITEND}'
EDLIB_CMD       = '{TIMELIMIT} {edlib_bin} {input} -p -s {TIMELIMITEND}'   # -p: Return alignment, -s: Silent / no output
WFA_CMD         = '{TIMELIMIT} {wfa_bin} -i {input} -a -a gap-affine-wfa  {TIMELIMITEND}'  # -p: Return alignment, -s: Silent / no output

ns = [100, 1000, 10000, 100000, 1000000]                    # number of pairs
N = 10000000                                                # total number of letters in all A sequences (half of all letters)
es = [0.01, 0.05, 0.20]                                     # error rate in [0;1]
algs = ['pa', 'edlib', 'wfa', 'pa-no-prune', 'dijkstra']
#algs = ['dijkstra']

# Params for PA
# (n, e) -> k        , where `k` is seed length
PARAMS = {
    (100, 0.01): 5,
    (100, 0.05): 5,
    (100, 0.20): 5,

    (1000, 0.01): 7,
    (1000, 0.05): 7,
    (1000, 0.20): 7,

    (10000, 0.01): 8,
    (10000, 0.05): 8,
    (10000, 0.20): 8,

    (100000, 0.01): 9,
    (100000, 0.05): 9,
    (100000, 0.20): 9,

    (1000000, 0.01): 12,
    (1000000, 0.05): 12,
    (1000000, 0.20): 9,
}

rule generate_all:
    input:
        # `x` number of pairs, each of length `n` and error rate `e`
        expand("data/input/{n[1]}x-n{n[0]}-e{e}.seq", n=[(n, N//n) for n in ns], e=es)

rule run_all:
    input:
        expand("data/runs/{n[1]}x-n{n[0]}-e{e}.{alg}.bench", n=[(n, N//n) for n in ns], e=es, alg=algs)

rule generate_data: 
    output:
        "data/input/{cnt}x-n{n}-e{e}.seq"
    shell:
        "../wfa/bin/generate_dataset -n {wildcards.cnt} -l {wildcards.n} -e {wildcards.e} -o {output}"

rule run_pairwise_aligner:
    input:
        "data/input/{cnt}x-n{n}-e{e}.seq"
    benchmark:
        "data/runs/{cnt}x-n{n}-e{e}.pa.bench"
    params:
        l = lambda w: PARAMS[(int(w.n),float(w.e))]
    shell:
        PA_CMD

rule run_pairwise_aligner_no_prune:
    input:
        "data/input/{cnt}x-n{n}-e{e}.seq"
    benchmark:
        "data/runs/{cnt}x-n{n}-e{e}.pa-no-prune.bench"
    params:
        l = lambda w: PARAMS[(int(w.n),float(w.e))]
    shell:
        PA_NO_PRUNE_CMD

rule run_dijkstra:
    input:
        "data/input/{cnt}x-n{n}-e{e}.seq"
    benchmark:
        "data/runs/{cnt}x-n{n}-e{e}.dijkstra.bench"
    shell:
        DIJKSTRA_CMD

rule run_edlib:
    input:
        "data/input/{cnt}x-n{n}-e{e}.seq"
    benchmark:
        "data/runs/{cnt}x-n{n}-e{e}.edlib.bench"
    params:
        l = lambda w: PARAMS[(int(w.n),float(w.e))]
    shell:
        EDLIB_CMD 

rule run_wfa:
    input:
        "data/input/{cnt}x-n{n}-e{e}.seq"
    benchmark:
        "data/runs/{cnt}x-n{n}-e{e}.wfa.bench"
    params:
        l = lambda w: PARAMS[(int(w.n),float(w.e))]
    shell:
        WFA_CMD

def try_read_files(paths):
    def f(path):
        try:
            return Path(path).read_text().strip()+'\t'
        except:
            return '\t'
    return [f(path) for path in paths]

# Collect all .benchfiles into a single tsv.
headers       = "alg\tcnt\tn\te\tband\ts\th:m:s\tmax_rss\tmax_vms\tmax_uss\tmax_pss\tio_in\tio_out\tmean_load\tcpu_time"
prefix       = "{alg}\t{n[1]}\t{n[0]}\t{e}"
rule run_benchmark:
    input:
        file = expand("data/runs/{n[1]}x-n{n[0]}-e{e}.{alg}.bench", n=[(n, N//n) for n in ns], e=es, alg=algs)
    output:
        f"data/benchmark_{N}.tsv"
    params:
        prefix = expand(prefix, n=[(n, N//n) for n in ns], e=es, alg=algs),
        band = try_read_files(expand("data/runs/{n[1]}x-n{n[0]}-e{e}.{alg}.bench.band", n=[(n, N//n) for n in ns], e=es, alg=algs))
    shell:
        "paste <(echo \"{params.prefix}\" | tr ' ' '\n') <(echo \"{params.band}\" | tr ' ' '\n') <(tail -n 1 --silent {input.file}) | sed '1s/^/{headers}\\n/' > {output}"


# Visualizations

rule astar_visualization:
    input:
    shell:
        "cargo run --release --example states"
