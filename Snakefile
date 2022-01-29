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

# INPUT DATA

# total number of letters in all A sequences (half of all letters)
# TODO: Scale this up.
N = 1_000_000
# Length of each sequence
# TODO: Add a test with 10M long sequences.
ns = [100, 300, 1_000, 3_000, 10_000, 30_000, 100_000, 300_000, 1_000_000]
# Error rate in [0;1]
es = [0.01, 0.05, 0.20]

# PA PARAMS

# Seed length, match distance pairs
kms = [(6,0), (7,0), (8,0), (9,0), (10, 0), (11, 0), (15, 0), (20, 0), (32, 0),
      (7, 1), (8, 1), (9, 1), (10, 1), (11, 1), (14, 1)]
# Prune fractions.
pfs = [0.01, 0.4, 0.8, 1.0]


# RUN SETTINGS

TIMEOUT = "10s"

REPEATS = 2

# Tools to run
algs = ['pa', 'edlib', 'wfa', 'dijkstra']

# TOOL DEFINITIONS

pa_bin    = 'target/release/pairwise-aligner'
edlib_bin = '../edlib/build/bin/edlib-aligner'
wfa_bin   = '../wfa/bin/align_benchmark'

TIMELIMIT       = f'(timeout {TIMEOUT}'
TIMELIMITEND    = ') || true'
# Run PA
PA_CMD          = '{TIMELIMIT} {pa_bin} -i {input} -o data/runs/x{wildcards.cnt}-n{wildcards.n}-e{wildcards.e}-k{wildcards.k}-m{wildcards.m}-pf{wildcards.pf}.pa.band -k {wildcards.k} -m {wildcards.m} --prune-fraction {wildcards.pf} --silent2 {TIMELIMITEND}'
# Run PA with as Dijkstra, using a heuristic that's always 0.
DIJKSTRA_CMD    = '{TIMELIMIT} {pa_bin} -i {input} -a Dijkstra --silent2 {TIMELIMITEND}'
# -p: Return alignment
# -s: Silent / no output
EDLIB_CMD       = '{TIMELIMIT} {edlib_bin} {input} -p -s {TIMELIMITEND}'
# -a: Algorithm to run
# --affine-penalties: Use edit distance score, with gap-opening cost of 0.
WFA_CMD         = '{TIMELIMIT} {wfa_bin} -i {input} -a gap-affine-wfa --affine-penalties="0,1,0,1" {TIMELIMITEND}'

# INPUT DATA RULES

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
        repeat("data/runs/x{cnt}-n{n}-e{e}-k{k}-m{m}-pf{pf}.pa.bench", REPEATS)
    shell:
        PA_CMD

# Collect all .bench files into a single tsv.
rule run_benchmark_params:
    input:
         expand("data/runs/x{n[0]}-n{n[1]}-e{e}-k{km[0]}-m{km[1]}-pf{pf}.{alg}.bench", n=[(N//n, n) for n in ns], e=es, km=kms, pf=pfs, alg=['pa'])
    output:
        f"data/table/params_N{N}.tsv"
    run:
        headers = ["alg","cnt","n","e", "k", "m", "pf", "band","s","h:m:s","max_rss","max_vms","max_uss","max_pss","io_in","io_out","mean_load","cpu_time"]
        table_file = Path(output[0]).open('w')
        table_file.write("\t".join(headers) + '\n')
        import itertools
        for ((cnt, n), e, (k, m), pf, alg) in itertools.product(((N//n, n) for n in ns), es, kms, pfs, ['pa']):
            bench_file = Path(f"data/runs/x{cnt}-n{n}-e{e}-k{k}-m{m}-pf{pf}.{alg}.bench")
            try:
                band = bench_file.with_suffix('.band').read_text().strip()
            except:
                band = ''
            for line in bench_file.read_text().splitlines()[1:]:
                table_file.write(f'{alg}\t{cnt}\t{n}\t{e}\t{k}\t{m}\t{pf}\t{band}\t{line}\n')


def average_bench_time(f):
    import statistics
    return statistics.mean(map(lambda line: float(line.split('\t')[0]), Path(f).read_text().splitlines()[1:]))

# Find the run with the smallest average benchmark time and copy those files.
rule find_best_runs:
    input:
        lambda w:
            expand("data/runs/x{wildcards.cnt}-n{wildcards.n}-e{wildcards.e}-k{km[0]}-m{km[1]}-pf{pf}.pa.bench", km=kms, pf=pfs, alg=['pa'], wildcards=[w])
    output:
        "data/runs/x{cnt}-n{n}-e{e,[0-9.]+}.pa.bench"
    run:
        # Loop over input bench files, find the one with the best average runtime, and copy it.
        (_, best_f) = min(map(lambda f: (average_bench_time(f), f), input))
        import shutil
        shutil.copy(best_f, output[0])


rule run_dijkstra:
    input:
        "data/input/x{cnt}-n{n}-e{e}.seq"
    benchmark:
        repeat("data/runs/x{cnt}-n{n}-e{e}.dijkstra.bench", REPEATS)
    shell:
        DIJKSTRA_CMD

rule run_edlib:
    input:
        "data/input/x{cnt}-n{n}-e{e}.seq"
    benchmark:
        repeat("data/runs/x{cnt}-n{n}-e{e}.edlib.bench", REPEATS)
    shell:
        EDLIB_CMD 

rule run_wfa:
    input:
        "data/input/x{cnt}-n{n}-e{e}.seq"
    benchmark:
        repeat("data/runs/x{cnt}-n{n}-e{e}.wfa.bench", REPEATS)
    shell:
        WFA_CMD

# Collect all .bench files into a single tsv.
rule run_benchmark_tools:
    input:
        expand("data/runs/x{n[0]}-n{n[1]}-e{e}.{alg}.bench", n=[(N//n, n) for n in ns], e=es, alg=algs)
    output:
        f"data/table/tools_N{N}.tsv"
    run:
        headers = ["alg","cnt","n","e","s","h:m:s","max_rss","max_vms","max_uss","max_pss","io_in","io_out","mean_load","cpu_time"]
        table_file = Path(output[0]).open('w')
        table_file.write("\t".join(headers) + '\n')
        import itertools
        for ((cnt, n), e, alg) in itertools.product(((N//n, n) for n in ns), es, algs):
            bench_file = Path(f"data/runs/x{cnt}-n{n}-e{e}.{alg}.bench")
            for line in bench_file.read_text().splitlines()[1:]:
                table_file.write(f'{alg}\t{cnt}\t{n}\t{e}\t{line}\n')

# Visualizations

rule astar_visualization:
    input:
    shell:
        "cargo run --release --example states"
