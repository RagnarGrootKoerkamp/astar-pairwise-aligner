ns = [100, 1000, 10000, 100000, 1000000]
N = 10000000
es = [0.01, 0.05, 0.20]

rule all_data:
    input:
        expand("data/{n[1]}x-n{n[0]}-e{e}.seq", n=[(n, N/n) for n in ns], e=es)

rule generate_data:
    output:
        "data/{cnt}x-n{n}-e{e}.seq"
    shell:
        "../wfa/bin/generate_dataset -n {wildcards.cnt} -l {wildcards.n} -e {wildcards.e} -o {output}"

rule astar_visualization:
    input:
    shell:
        "cargo run --release --example states"
