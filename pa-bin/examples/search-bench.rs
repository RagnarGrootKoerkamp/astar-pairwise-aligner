use std::hint::black_box;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Result {
    pattern_len: usize,
    text_len: usize,
    trace: bool,
    dist: f32,
    k: i32,
    search: u128,
    edlib: u128,
}

fn main() {
    let mut results = Vec::new();

    for trace in [false] {
        for k in [-1, 100000] {
            for dist in [0.01, 0.1, 1.0] {
                test(trace, dist, k, &mut results);
            }
        }
    }

    serde_json::to_writer(std::io::stdout(), &results).unwrap();
}

fn test(trace: bool, dist: f32, k: i32, results: &mut Vec<Result>) {
    eprintln!("trace: {trace}, dist: {dist} k: {k}");
    let config = edlib_rs::EdlibAlignConfigRs {
        k,
        mode: edlib_rs::EdlibAlignModeRs::EDLIB_MODE_HW,
        task: if trace {
            edlib_rs::EdlibAlignTaskRs::EDLIB_TASK_PATH
        } else {
            edlib_rs::EdlibAlignTaskRs::EDLIB_TASK_DISTANCE
        },
        additionalequalities: &[],
    };

    for pattern_len in [32, 64, 128, 256, 512, 1024, 2048] {
        for text_len in [50000] {
            if text_len < pattern_len {
                continue;
            }
            let pattern = pa_generate::random_seq(pattern_len);
            let p2 = pa_generate::mutate(
                &pattern,
                (dist * pattern_len as f32) as usize,
                &mut rand::rng(),
            );
            let mut text = pa_generate::random_seq(text_len);
            let l = (text.len() - pattern.len()) / 2;
            text.splice(l..l + p2.len(), p2.clone());

            let (t_search, (r1, _trace)) = if trace {
                time(|| {
                    let r = pa_bitpacking::search::search(&pattern, &text, 1.0);
                    let minpos = r.out.iter().enumerate().min_by_key(|x| x.1).unwrap().0;
                    let trace = r.trace(minpos);
                    (r, trace)
                })
            } else {
                time(|| {
                    (
                        pa_bitpacking::search::search(&pattern, &text, 1.0),
                        Default::default(),
                    )
                })
            };
            black_box(_trace);

            let (t_edlib, r2) = time(|| edlib_rs::edlibAlignRs(&pattern, &text, &config));

            let value = Result {
                pattern_len,
                text_len,
                trace,
                dist,
                k,
                search: t_search.as_nanos(),
                edlib: t_edlib.as_nanos(),
            };
            results.push(value);
            eprint!(
                "{:>5} {:>5} {:3.1}x  | ",
                t_search.as_micros(),
                t_edlib.as_micros(),
                t_edlib.as_secs_f32() / t_search.as_secs_f32()
            );

            let d1 = *r1.out.iter().min().unwrap();
            let d2 = r2.editDistance;
            // assert_eq!(d1, d2, "Failed at {} {}", pattern_len, text_len);
        }
    }
    eprintln!();
}

fn time<T>(mut f: impl FnMut() -> T) -> (std::time::Duration, T) {
    let mut times = vec![];
    let mut r = f();
    // warmup
    for _ in 0..100 {
        black_box(f());
    }
    for _ in 0..11 {
        let start = std::time::Instant::now();
        r = f();
        let elapsed = start.elapsed();
        times.push(elapsed);
    }
    times.sort();
    let avg = (times[4] + times[5] + times[6]) / 3;
    (avg, r)
}
