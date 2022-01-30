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

## INPUT DATA

# total number of letters in all A sequences (half of all letters)
N = 100_000
# Length of each sequence
ns = [100, 300, 1_000, 3_000, 10_000, 30_000, 100_000, 300_000, 1_000_000, 3_000_000, 10_000_000]
# Error rate in [0;1]
es = [0.01, 0.05, 0.20]

## PA PARAMS

# Seed length, match distance pairs
kms = [(6,0), (8,0), (10, 0), (12, 0), (15, 0), (20, 0), (32, 0),
      (7, 1), (9, 1), (11, 1), (14, 1)]
# Prune fractions.
pfs = [0.01, 0.4, 0.8, 1.0]

## TOOLS

# 'dijkstra' is slow
algs = ['pa', 'edlib', 'wfa']

## RUN SETTINGS

TIMEOUT = "10s"
REPEATS = 1

## TOOL DEFINITIONS

pa_bin    = 'target/release/pairwise-aligner'
edlib_bin = '../edlib/build/bin/edlib-aligner'
wfa_bin   = '../wfa/bin/align_benchmark'

TIMELIMIT       = f'(timeout {TIMEOUT}'
TIMELIMITEND    = ') || true'

# Generate testcases
GENERATE_CMD    = '../wfa/bin/generate_dataset -n {wildcards.cnt} -l {wildcards.n} -e {wildcards.e} -o {output}'
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

## WRAPPER CLASSES

class Input:
    def __init__(self, **kwargs):
        self.cnt = int(kwargs['cnt'])
        self.n = int(kwargs['n'])
        self.e = float(kwargs['e'])
    def name_pattern():
        return 'x{cnt}-n{n}-e{e}'
    def pattern():
        return 'data/input/x{cnt}-n{n}-e{e,[0-9.]+}.seq'
    def name(self):
        return Path(f'x{self.cnt}-n{self.n}-e{self.e}')
    def path(self):
        return Path(f'data/input/{self.name()}.seq')

inputs = [Input(cnt=N//n, n=n, e=e) for n in ns for e in es]

class Run:
    def __init__(self, **kwargs):
        self.__dict__.update(kwargs)
    def pattern_with_params():
        alg = 'pa'
        return f'data/runs/{Input.name_pattern()}-A{alg}-m{{m}}-k{{k}}-pf{{pf}}.bench'
    def pattern(alg='pa'):
        return f'data/runs/{Input.name_pattern()}-A{alg}.bench'
    def bench_path_with_params(self):
        return Path(f'data/runs/{self.input.name()}-A{self.alg}-m{self.m}-k{self.k}-pf{self.pf}.bench')
    def bench_path(self):
        return Path(f'data/runs/{self.input.name()}-A{self.alg}.bench')
    def band_path(self):
        assert self.alg == "pa"
        return self.bench_path().with_suffix('.band')

# Returns false for parameters that should be skipped.
def run_filter(run):
    if run.alg != 'pa': return True
    # If the error rate is larger than the max rate this (k,m) can handle, skip.
    # e is made a bit lower, since an induced error rate of 20% only gives ~15% edit distance.
    max_error_rate = (run.m+1) / run.k
    if run.input.e * 0.8 > max_error_rate:
        return False

    # Don't do inexact matches for low error rates.
    if run.input.e < 0.10 and run.input.m > 0:
        return False

    # If we expect more than 64 matches per seed, skip since we should increase k instead.
    if 4**run.k < run.input.n / 64:
        return False

    # Never do partial pruning for m=1
    if run.m == 1 and run.pf < 1:
        return False

    return True

pa_runs = filter(run_filter, (Run(input=input, alg='pa', m=m, k=k, pf=pf)
                        for input in inputs
                        for (k, m) in kms
                        for pf in pfs))
def pa_runs_for_input(input):
    return filter(run_filter, (Run(input=input, alg='pa', m=m, k=k, pf=pf)
                        for (k, m) in kms
                        for pf in pfs))
tool_runs = [Run(input=input, alg=alg)
                for input in inputs
                for alg in algs]

## INPUT DATA RULES

rule generate_input:
    output: Input.pattern()
    shell: GENERATE_CMD

## PA WITH PARAMS

rule run_pairwise_aligner:
    input: lambda w: Input(**w).path()
    benchmark: repeat(Run.pattern_with_params(), REPEATS)
    shell: PA_CMD

# Collect all .bench files into a single tsv.
rule params_table:
    input: [run.bench_path() for run in pa_runs]
    output: f"data/table/params_N{N}.tsv"
    run:
        headers = ["alg","cnt","n","e", "k", "m", "pf", "band","s","h:m:s","max_rss","max_vms","max_uss","max_pss","io_in","io_out","mean_load","cpu_time"]
        table_file = Path(output[0]).open('w')
        table_file.write("\t".join(headers) + '\n')
        import itertools
        for run in pa_runs:
            try:
                band = run.band_path().read_text().strip()
            except:
                band = ''
            for line in run.bench_path().read_text().splitlines()[1:]:
                table_file.write(f'{alg}\t{cnt}\t{n}\t{e}\t{k}\t{m}\t{pf}\t{band}\t{line}\n')


def average_bench_time(f):
    import statistics
    return statistics.mean(map(lambda line: float(line.split('\t')[0]), Path(f).read_text().splitlines()[1:]))

# Find the run with the smallest average benchmark time and copy those files.
rule find_best_runs:
    input: lambda w: [run.bench_path_with_params() for run in pa_runs_for_input(Input(**w))]
    output: Run.pattern()
    run:
        # Loop over input bench files, find the one with the best average runtime, and copy it.
        (_, best_f) = min(map(lambda f: (average_bench_time(f), f), input))
        import shutil
        shutil.copy(best_f, output[0])

## RUN OTHER TOOLS

rule run_dijkstra:
    input: lambda w: Input(**w).path()
    benchmark: repeat(Run.pattern('dijkstra'), REPEATS)
    shell: DIJKSTRA_CMD

rule run_edlib:
    input: lambda w: Input(**w).path()
    benchmark: repeat(Run.pattern('edlib'), REPEATS)
    shell: EDLIB_CMD

rule run_wfa:
    input: lambda w: Input(**w).path()
    benchmark: repeat(Run.pattern('wfa'), REPEATS)
    shell: WFA_CMD

# Collect all .bench files into a single tsv.
rule tools_table:
    input: [run.bench_path() for run in tool_runs]
    output: f"data/table/tools_N{N}.tsv"
    run:
        headers = ["alg","cnt","n","e","s","h:m:s","max_rss","max_vms","max_uss","max_pss","io_in","io_out","mean_load","cpu_time"]
        table_file = Path(output[0]).open('w')
        table_file.write("\t".join(headers) + '\n')
        import itertools
        for run in tool_runs:
            for line in run.bench_path().read_text().splitlines()[1:]:
                table_file.write(f'{run.alg}\t{run.input.cnt}\t{run.input.n}\t{run.input.e}\t{line}\n')

# Visualizations

rule astar_visualization:
    input:
    shell: "cargo run --release --example states"
