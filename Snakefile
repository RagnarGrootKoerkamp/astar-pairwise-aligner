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
#   - Clone the fork, which contains a fix to allow setting parameters to mimick edit distance.
#   - git clone https://github.com/RagnarGrootKoerkamp/WFA wfa
#   - cd wfa
#   - make clean all
#
# Instructions for running:
# - snakemake -p -c all run_benchmark

pa_bin    = "target/release/pairwise-aligner"
edlib_bin = "../edlib/build/bin/edlib-aligner"
wfa_bin   = "../wfa/bin/align_benchmark"

TIMELIMIT       = "(timeout 6s"
TIMELIMITEND    = ") || true"

REPEATS = 2

# Run PA
PA_CMD          = '{TIMELIMIT} {pa_bin} -i {input} -o data/runs/x{wildcards.cnt}-n{wildcards.n}-e{wildcards.e}-k{wildcards.k}-m{wildcards.m}-pf{wildcards.pf}.pa.{wildcards.sample}.band -k {wildcards.k} -m {wildcards.m} --prune-fraction {wildcards.pf} --silent2 {TIMELIMITEND}'
# Run PA with as Dijkstra, using a heuristic that's always 0.
DIJKSTRA_CMD    = '{TIMELIMIT} {pa_bin} -i {input} -a Dijkstra --silent2 {TIMELIMITEND}'
# Run PA given optimal parameters.
PA_OPTIMAL_CMD          = '{TIMELIMIT} {pa_bin} -i {input} -k {params.k}  -m {params.m} --prune-fraction {params.pf} --silent2 {TIMELIMITEND}'
# -p: Return alignment
# -s: Silent / no output
EDLIB_CMD       = '{TIMELIMIT} {edlib_bin} {input} -p -s {TIMELIMITEND}'
# -a: Algorithm to run
# --affine-penalties: Use edit distance score, with gap-opening cost of 0.
WFA_CMD         = '{TIMELIMIT} {wfa_bin} -i {input} -a gap-affine-wfa --affine-penalties="0,1,0,1" {TIMELIMITEND}'

# total number of letters in all A sequences (half of all letters)
# TODO: Scale this up.
N = 1_000_000
# number of pairs
# TODO: Add a test with 10M long sequences.
ns = [100, 1_000, 10_000, 100_000, 1_000_000]
# error rate in [0;1]
es = [0.01, 0.05, 0.20]
# seed length, match distance pairs
kms = [(6,0), (7,0), (8,0), (9,0), (10, 0), (11, 0), (15, 0), (20, 0), (32, 0),
      (7, 1), (8, 1), (9, 1), (10, 1), (11, 1), (14, 1)]
# Prune fractions.
pfs = [0.01, 0.4, 0.8, 1.0]

algs = ['pa', 'edlib', 'wfa', 'dijkstra']

# Params for PA
# (n, e) -> k        , where `k` is seed length
# TODO: Add m and prune-fraction to this.
OPTIMAL_PARAMS = {
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
        expand("data/input/x{n[1]}-n{n[0]}-e{e}.seq", n=[(n, N//n) for n in ns], e=es)

rule generate_data:
    output:
        "data/input/x{cnt}-n{n}-e{e}.seq"
    shell:
        "../wfa/bin/generate_dataset -n {wildcards.cnt} -l {wildcards.n} -e {wildcards.e} -o {output}"

# COMPARISON OF VARIOUS PARAMETERS OF PA
rule run_pairwise_aligner:
    input:
        "data/input/x{cnt}-n{n}-e{e}.seq"
    benchmark:
        repeat("data/runs/x{cnt}-n{n}-e{e}-k{k}-m{m}-pf{pf}.pa.{sample}.bench", REPEATS)
    shell:
        PA_CMD


# COMPARISON WITH OTHER TOOLS
rule run_all_optimal:
    input:
        expand("data/runs/x{n[1]}-n{n[0]}-e{e}.{alg}.{sample}.bench", n=[(n, N//n) for n in ns], e=es, alg=algs, sample=range(REPEATS))

# Collect all .benchfiles into a single tsv.
def try_read_files(paths):
    def f(path):
        try:
            return Path(path).read_text().strip()
        except:
            return ''
    return [f(path) for path in paths]

headers_params       = "alg\tcnt\tn\te\tk\tm\tpf\tsample\tband\ts\th:m:s\tmax_rss\tmax_vms\tmax_uss\tmax_pss\tio_in\tio_out\tm_load\tcpu_time"
prefix_params       = "{alg}\t{n[1]}\t{n[0]}\t{e}\t{km[0]}\t{km[1]}\t{pf}\t{sample}"
rule run_benchmark_params:
    input:
        file = expand("data/runs/x{n[1]}-n{n[0]}-e{e}-k{km[0]}-m{km[1]}-pf{pf}.{alg}.{sample}.bench", n=[(n, N//n) for n in ns], e=es, km=kms, pf=pfs, alg=['pa'], sample=range(REPEATS))
    output:
        f"data/table/params_N{N}.tsv"
    params:
        prefix = expand(prefix_params, n=[(n, N//n) for n in ns], e=es, km=kms, pf=pfs, alg=['pa'], sample=range(REPEATS)),
        band = try_read_files(expand("data/runs/x{n[1]}-n{n[0]}-e{e}-k{km[0]}-m{km[1]}-pf{pf}.{alg}.{sample}.band", n=[(n, N//n) for n in ns], e=es, km=kms, pf=pfs, alg=['pa'], sample=range(REPEATS)))
    shell:
        "paste <(echo \"{params.prefix}\" | tr ' ' '\n') <(echo \"{params.band}\" | tr ' ' '\n') <(tail -n 1 --silent {input.file}) | sed '1s/^/{headers_params}\\n/' > {output}"

# Run PA with parameters from OPTIMAL_PARAMETERS, for comparison with other tool.
rule run_pairwise_aligner_optimal:
    input:
        "data/input/x{cnt}-n{n}-e{e}.seq"
    benchmark:
        repeat("data/runs/x{cnt}-n{n}-e{e,[0-9.]+}.pa.{sample}.bench", REPEATS)
    params:
        k = lambda w: OPTIMAL_PARAMS[(int(w.n),float(w.e))],
        m = 0,
        pf = 1.0
    shell:
        PA_OPTIMAL_CMD

rule run_dijkstra:
    input:
        "data/input/x{cnt}-n{n}-e{e}.seq"
    benchmark:
        repeat("data/runs/x{cnt}-n{n}-e{e}.dijkstra.{sample}.bench", REPEATS)
    shell:
        DIJKSTRA_CMD

rule run_edlib:
    input:
        "data/input/x{cnt}-n{n}-e{e}.seq"
    benchmark:
        repeat("data/runs/x{cnt}-n{n}-e{e}.edlib.{sample}.bench", REPEATS)
    shell:
        EDLIB_CMD 

rule run_wfa:
    input:
        "data/input/x{cnt}-n{n}-e{e}.seq"
    benchmark:
        repeat("data/runs/x{cnt}-n{n}-e{e}.wfa.{sample}.bench", REPEATS)
    shell:
        WFA_CMD

# Collect all .benchfiles into a single tsv.
headers       = "alg\tcnt\tn\te\ts\th:m:s\tmax_rss\tmax_vms\tmax_uss\tmax_pss\tio_in\tio_out\tmean_load\tcpu_time"
prefix       = "{alg}\t{n[1]}\t{n[0]}\t{e}"
rule run_benchmark_tools:
    input:
        file = expand("data/runs/x{n[1]}-n{n[0]}-e{e}.{alg}.{sample}.bench", n=[(n, N//n) for n in ns], e=es, alg=algs, sample=range(REPEATS))
    output:
        f"data/table/tools_N{N}.tsv"
    params:
        prefix = expand(prefix, n=[(n, N//n) for n in ns], e=es, alg=algs, sample=range(REPEATS)),
    shell:
        "paste <(echo \"{params.prefix}\" | tr ' ' '\n') <(tail -n 1 --silent {input.file}) | sed '1s/^/{headers}\\n/' > {output}"


# Visualizations

rule astar_visualization:
    input:
    shell:
        "cargo run --release --example states"
