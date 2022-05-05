use pairwise_aligner::prelude::*;

fn main() {
    let n = 100;
    let e = 0.2;

    let m = 0;
    let k = 4;

    let (ref a, ref b, ref alphabet, stats) = setup(n, e);

    let target = Pos::from_length(&a, &b);
    let hmax = Some(align(a, b, alphabet, stats, ZeroCost).edit_distance);

    {
        let h = ZeroCost;
        let h = Heuristic::build(&h, &a, &b, &alphabet);
        h.display(target, DisplayType::Heuristic, hmax);
        //h.display(target, DisplayType::Contours, hmax);
    }
    {
        let h = GapCost;
        let h = Heuristic::build(&h, &a, &b, &alphabet);
        h.display(target, DisplayType::Heuristic, hmax);
        //h.display(target, DisplayType::Contours, hmax);
    }
    {
        let h = CountCost;
        let h = Heuristic::build(&h, &a, &b, &alphabet);
        h.display(target, DisplayType::Heuristic, hmax);
        //h.display(target, DisplayType::Contours, hmax);
    }
    {
        let h = SH {
            match_config: MatchConfig {
                length: Fixed(k),
                max_match_cost: m,
                ..MatchConfig::default()
            },
            pruning: false,
        };
        let h = Heuristic::build(&h, &a, &b, &alphabet);
        h.display(target, DisplayType::Heuristic, hmax);
        h.display(target, DisplayType::Contours, hmax);
    }
    {
        let h = CSH {
            match_config: MatchConfig {
                length: Fixed(k),
                max_match_cost: m,
                ..MatchConfig::default()
            },
            pruning: true,
            use_gap_cost: false,
            c: PhantomData::<HintContours<BruteForceContour>>::default(),
        };
        let h = Heuristic::build(&h, &a, &b, &alphabet);
        h.display(target, DisplayType::Heuristic, hmax);
        h.display(target, DisplayType::Contours, hmax);
    }
    {
        let h = CSH {
            match_config: MatchConfig {
                length: Fixed(k),
                max_match_cost: m,
                ..MatchConfig::default()
            },
            pruning: true,
            use_gap_cost: true,
            c: PhantomData::<HintContours<BruteForceContour>>::default(),
        };
        let h = Heuristic::build(&h, &a, &b, &alphabet);
        h.display(target, DisplayType::Heuristic, hmax);
        h.display(target, DisplayType::Contours, hmax);
    }
}
