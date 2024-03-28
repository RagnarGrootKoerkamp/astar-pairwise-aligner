use super::*;
use pa_heuristic::*;
use pa_test::*;
use pa_vis::NoVis;

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
fn band_doubling_gapgap() {
    test_aligner(AstarPa2 {
        doubling: DoublingType::band_doubling(),
        domain: Domain::gap_gap(),
        block_width: 64,
        ..nw()
    });
}

#[test]
fn dt_trace_gapgap() {
    test_aligner(AstarPa2 {
        doubling: DoublingType::band_doubling(),
        domain: Domain::gap_gap(),
        block_width: 256,
        block: BlockParams {
            dt_trace: true,
            ..Default::default()
        },
        ..nw()
    })
}

#[test]
fn band_doubling_dijkstra() {
    test_aligner(AstarPa2 {
        doubling: DoublingType::band_doubling(),
        domain: Domain::dijkstra(),
        block_width: 64,
        ..nw()
    });
}

#[test]
fn band_doubling_edlib() {
    test_aligner(AstarPa2 {
        doubling: DoublingType::band_doubling(),
        domain: Domain::dist_gap(),
        block_width: 64,
        ..nw()
    });
}

#[test]
fn band_doubling() {
    test_aligner(AstarPa2 {
        doubling: DoublingType::band_doubling(),
        domain: Domain::Astar(SH {
            match_config: MatchConfig::exact(5),
            pruning: Pruning::disabled(),
        }),
        block_width: 1,
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
fn incremental_doubling() {
    test_aligner(AstarPa2 {
        doubling: DoublingType::band_doubling(),
        domain: Domain::dist_gap(),
        block_width: 64,
        block: BlockParams {
            dt_trace: true,
            incremental_doubling: true,
            ..Default::default()
        },
        ..nw()
    });
}

#[test]
#[ignore = "local doubling is broken"]
fn local_doubling() {
    test_aligner(AstarPa2 {
        doubling: DoublingType::LocalDoubling,
        domain: Domain::Astar(GCSH::new(MatchConfig::exact(15), Pruning::start())),
        block_width: 256,
        ..nw()
    })
}
