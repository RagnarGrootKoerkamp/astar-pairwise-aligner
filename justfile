
codecoverage:
   CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' \
       cargo test -p astarpa2
   grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage
   rm *.profraw **/*.profraw
   open target/coverage/html/index.html

vis:
   cargo run -r --example aligners_vis --features sdl
