fn main() {
    let pattern = pa_generate::random_seq(100);
    let mut text = pa_generate::random_seq(400);

    let p1 = pa_generate::mutate(&pattern, 10, &mut rand::rng());
    let p2 = pa_generate::mutate(&pattern, 20, &mut rand::rng());
    let p3 = pa_generate::mutate(&pattern, 30, &mut rand::rng());

    text.splice(0..0, p1[50..].iter().copied());
    text.splice(200..200, p2);
    let l = text.len();
    text.splice(l..l, p3[..50].iter().copied());

    for u in [0.0, 0.5, 1.0] {
        let result = pa_bitpacking::search::search(&pattern, &text, u);
        println!("{:?}", result.out);
    }
}
