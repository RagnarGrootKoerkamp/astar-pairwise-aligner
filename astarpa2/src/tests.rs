use super::*;
use pa_heuristic::*;
use pa_test::test_aligner;
use pa_vis_types::NoVis;

fn nw() -> AstarPa2<NoVis, NoCost> {
    AstarPa2 {
        doubling: DoublingType::None,
        domain: Domain::full(),
        block_width: 1,
        v: NoVis,
        block: BlockParams::default(),
        trace: true,
        sparse_h: true,
        prune: true,
    }
}

#[test]
fn full() {
    test_aligner(nw());
}

#[test]
fn band_doubling() {
    test_aligner(AstarPa2 {
        doubling: DoublingType::band_doubling(),
        domain: Domain::Astar(SH {
            match_config: MatchConfig::exact(5),
            pruning: Pruning::disabled(),
        }),
        ..nw()
    });
}

#[test]
fn nw_prune() {
    test_aligner(AstarPa2 {
        doubling: DoublingType::band_doubling(),
        domain: Domain::Astar(GCSH::new(MatchConfig::exact(15), Pruning::start())),
        block_width: 256,
        ..nw()
    })
}

#[test]
fn dt_trace() {
    test_aligner(AstarPa2 {
        doubling: DoublingType::band_doubling(),
        domain: Domain::Astar(GCSH::new(MatchConfig::exact(15), Pruning::start())),
        block_width: 256,
        block: {
            let mut f = BlockParams::default();
            f.dt_trace = true;
            f
        },
        ..nw()
    })
}

#[test]
fn local_doubling() {
    test_aligner(AstarPa2 {
        doubling: DoublingType::LocalDoubling,
        domain: Domain::Astar(GCSH::new(MatchConfig::exact(15), Pruning::start())),
        block_width: 256,
        ..nw()
    })
}
