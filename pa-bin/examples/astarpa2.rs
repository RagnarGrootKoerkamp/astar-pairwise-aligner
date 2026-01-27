fn main() {
    let mut params = astarpa2::AstarPa2Params::full();
    params.front.incremental_doubling = false;
    let mut aligner = params.make_aligner(true);

    let a = b"ACTGCACACA";
    let b = b"ACGACGCATA";

    let (cost, cigar) = aligner.align(a, b);
    eprintln!("Cost: {}", cost);
    eprintln!("Cigar: {}", cigar.unwrap().to_string());
}
