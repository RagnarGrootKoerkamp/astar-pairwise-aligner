rule compare:
    input:
        "src/*"
    output:
        "evals/stats/table.csv"
    shell:
        "cargo run --release --example compare"

rule block_aligner:
    input:
    output:
        "evals/stats/table.csv"
    shell:
        "cargo run --release --example compare"

rule states:
    input:
    shell:
        "cargo run --release --example states"
