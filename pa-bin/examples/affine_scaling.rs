use pa_affine_types::AffineCost;
use pa_base_algos::nw::NW;
use pa_generate::{generate_model, ErrorModel};
use pa_heuristic::NoCost;
use pa_vis_types::NoVis;
use rand::{thread_rng, Rng};

fn main() {
    let rng = &mut thread_rng();
    let n = 1000;
    let error_models = [
        ErrorModel::Uniform,
        ErrorModel::NoisyInsert,
        ErrorModel::NoisyDelete,
        ErrorModel::SymmetricRepeat,
    ];
    let es = [0.01, 0.02, 0.05, 0.10, 0.20];
    let cost_models = [
        AffineCost::affine(1, 6, 2),
        AffineCost::affine(2, 6, 2),
        AffineCost::affine(3, 6, 2),
        AffineCost::affine(4, 6, 2),
        AffineCost::affine(4, 6, 3),
        AffineCost::affine(4, 6, 4),
    ];
    // Run each test on a new random seed for increased coverage over time.
    let seed = rng.gen_range(0..u64::MAX);
    for m in &error_models {
        eprintln!("\nError model {m:?}");
        for cm in &cost_models {
            eprintln!("Cost model {cm:?}");
            for e in es {
                let (ref a, ref b) = generate_model(n, e, *m, seed);
                let cost = NW {
                    cm: cm.clone(),
                    use_gap_cost_heuristic: false,
                    exponential_search: true,
                    local_doubling: false,
                    h: NoCost,
                    v: NoVis,
                }
                .cost(a, b);
                eprintln!(
                    "{e}: \t {}\t {}",
                    cost as f32 / n as f32,
                    cost as f32 / n as f32 / e as f32
                );
            }
        }
    }
}