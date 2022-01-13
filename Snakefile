ns = [100, 1000, 10000, 100000, 1000000]
N = 10000000
es = [0.01, 0.05, 0.20]
algs = ["pa", "edlib"]

pairwise_aligner_binary="target/release/pairwise-aligner"
edlib_binary="../edlib/build/bin/edlib-aligner"

# Map of parameters to use given length and edit distance.
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
        "data/input/{cnt}x-n{n}-e{e}.seq",
        pairwise_aligner_binary
    benchmark:
        "data/runs/{cnt}x-n{n}-e{e}.pa.bench"
    params:
        l = lambda w: PARAMS[(int(w.n),float(w.e))]
    shell:
        # -l: k-mer length
        # --silent: Silent / no output
        '{pairwise_aligner_binary} -i {input[0]} -l {params.l} --silent'


rule run_edlib:
    input:
        "data/input/{cnt}x-n{n}-e{e}.seq",
        edlib_binary
    benchmark:
        "data/runs/{cnt}x-n{n}-e{e}.edlib.bench"
    params:
        l = lambda w: PARAMS[(int(w.n),float(w.e))]
    shell:
        # -p: Return alignment
        # -s: Silent / no output
        '{edlib_binary} {input[0]} -p -s'


# Collect all .benchfiles into a single csv.
headers       = "alg\tcnt\tn\te\ts\th:m:s\tmax_rss\tmax_vms\tmax_uss\tmax_pss\tio_in\tio_out\tmean_load"
prefix       = "{alg}\t{n[1]}\t{n[0]}\t{e}"
rule benchmark:
    input:
        file = expand("data/runs/{n[1]}x-n{n[0]}-e{e}.{alg}.bench", n=[(n, N//n) for n in ns], e=es, alg=algs)
    output:
        f"data/benchmark_{N}.csv"
    params:
        prefix = expand(prefix, n=[(n, N//n) for n in ns], e=es, alg=algs)
    shell:
        "paste <(echo \"{params.prefix}\" | tr ' ' '\n') <(tail -n 1 --silent {input.file}) | sed '1s/^/{headers}\\n/' > {output}"


# Visualizations

rule astar_visualization:
    input:
    shell:
        "cargo run --release --example states"
