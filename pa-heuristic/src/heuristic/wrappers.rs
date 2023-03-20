use super::*;
use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct EqualHeuristic<H1: Heuristic, H2: Heuristic> {
    pub h1: H1,
    pub h2: H2,
}

pub struct EqualHeuristicI<'a, H1: Heuristic, H2: Heuristic> {
    h1: H1::Instance<'a>,
    h2: H2::Instance<'a>,
}

impl<H1: Heuristic, H2: Heuristic> Heuristic for EqualHeuristic<H1, H2>
where
    for<'a> H1::Instance<'a>: HeuristicInstance<'a>,
    for<'a> H2::Instance<'a>: HeuristicInstance<'a>,
    Pos: Copy + Eq + std::fmt::Debug + Default,
{
    type Instance<'a> = EqualHeuristicI<'a, H1, H2>;

    fn name(&self) -> String {
        self.h1.name() + "+" + &self.h2.name()
    }

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::Instance<'a> {
        EqualHeuristicI {
            h1: self.h1.build(a, b),
            h2: self.h2.build(a, b),
        }
    }
}

impl<'a, H1: Heuristic, H2: Heuristic> HeuristicInstance<'a> for EqualHeuristicI<'a, H1, H2>
where
    H1::Instance<'a>: HeuristicInstance<'a>,
    H2::Instance<'a>: HeuristicInstance<'a>,
{
    fn h(&self, pos: Pos) -> Cost {
        let h1 = self.h1.h(pos);
        let h2 = self.h2.h(pos);
        // h1 is the slow accurate one, h2 the fast inaccurate one.
        assert!(
            h1 == h2,
            "\nValues differ at {:?}:\n{:?}\n ===> {}\n{:?}\n ===> {}\n",
            pos,
            self.h1.params_string(),
            h1,
            self.h2.params_string(),
            h2
        );
        h2
    }

    type Hint = (
        <<H1 as Heuristic>::Instance<'a> as HeuristicInstance<'a>>::Hint,
        <<H2 as Heuristic>::Instance<'a> as HeuristicInstance<'a>>::Hint,
    );

    fn seeds(&self) -> Option<&Seeds> {
        self.h2.seeds()
    }

    fn matches(&self) -> Option<Vec<Match>> {
        self.h2.matches()
    }

    fn is_seed_start_or_end(&self, pos: Pos) -> bool {
        let s1 = self.h1.is_seed_start_or_end(pos);
        let s2 = self.h2.is_seed_start_or_end(pos);
        assert_eq!(s1, s2);
        s2
    }

    fn prune(&mut self, pos: Pos, hint: Self::Hint) -> (Cost, ()) {
        let _c1 = self.h1.prune(pos, hint.0);
        let _c2 = self.h2.prune(pos, hint.1);
        (0, ())
    }

    fn h_with_hint(&self, pos: Pos, hint: Self::Hint) -> (Cost, Self::Hint) {
        let (c1, hint1) = self.h1.h_with_hint(pos, hint.0);
        let (c2, hint2) = self.h2.h_with_hint(pos, hint.1);
        assert!(
            c1 == c2,
            "\nValues differ at {:?}:\n{:?}\n ===> {}\n{:?}\n ===> {}\n",
            pos,
            self.h1.params_string(),
            c1,
            self.h2.params_string(),
            c2
        );
        (c2, (hint1, hint2))
    }

    fn stats(&mut self) -> HeuristicStats {
        self.h2.stats()
    }

    fn root_potential(&self) -> Cost {
        self.h2.root_potential()
    }

    fn explore(&mut self, pos: Pos) {
        self.h1.explore(pos);
        self.h2.explore(pos);
    }

    fn layer(&self, pos: Pos) -> Option<Cost> {
        self.h2.layer(pos)
    }

    fn layer_with_hint(&self, pos: Pos, hint: Self::Hint) -> Option<(Cost, Self::Hint)> {
        let (c, h) = self.h2.layer_with_hint(pos, hint.1)?;
        Some((c, (hint.0, h)))
    }

    fn h_with_parent(&self, pos: Pos) -> (Cost, Pos) {
        (self.h(pos), Pos::default())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MaxHeuristic<H1: Heuristic, H2: Heuristic> {
    pub h1: H1,
    pub h2: H2,
}

pub struct MaxHeuristicI<'a, H1: Heuristic, H2: Heuristic> {
    h1: H1::Instance<'a>,
    h2: H2::Instance<'a>,
}

impl<H1: Heuristic, H2: Heuristic> Heuristic for MaxHeuristic<H1, H2>
where
    for<'a> H1::Instance<'a>: HeuristicInstance<'a>,
    for<'a> H2::Instance<'a>: HeuristicInstance<'a>,
    Pos: Copy + Eq + std::fmt::Debug + Default,
{
    type Instance<'a> = MaxHeuristicI<'a, H1, H2>;

    fn name(&self) -> String {
        "max(".to_owned() + &self.h1.name() + "," + &self.h2.name() + ")"
    }

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::Instance<'a> {
        MaxHeuristicI {
            h1: self.h1.build(a, b),
            h2: self.h2.build(a, b),
        }
    }
}

impl<'a, H1: Heuristic, H2: Heuristic> HeuristicInstance<'a> for MaxHeuristicI<'a, H1, H2>
where
    H1::Instance<'a>: HeuristicInstance<'a>,
    H2::Instance<'a>: HeuristicInstance<'a>,
{
    fn h(&self, pos: Pos) -> Cost {
        let h1 = self.h1.h(pos);
        let h2 = self.h2.h(pos);
        max(h1, h2)
    }

    type Hint = (
        <<H1 as Heuristic>::Instance<'a> as HeuristicInstance<'a>>::Hint,
        <<H2 as Heuristic>::Instance<'a> as HeuristicInstance<'a>>::Hint,
    );

    fn is_seed_start_or_end(&self, pos: Pos) -> bool {
        let s1 = self.h1.is_seed_start_or_end(pos);
        let s2 = self.h2.is_seed_start_or_end(pos);
        s1 || s2
    }

    fn prune(&mut self, pos: Pos, hint: Self::Hint) -> (Cost, ()) {
        let _c1 = self.h1.prune(pos, hint.0);
        let _c2 = self.h2.prune(pos, hint.1);
        (0, ())
    }

    fn h_with_hint(&self, pos: Pos, hint: Self::Hint) -> (Cost, Self::Hint) {
        let (c1, hint1) = self.h1.h_with_hint(pos, hint.0);
        let (c2, hint2) = self.h2.h_with_hint(pos, hint.1);
        (max(c1, c2), (hint1, hint2))
    }

    fn stats(&mut self) -> HeuristicStats {
        self.h2.stats()
    }

    fn root_potential(&self) -> Cost {
        max(self.h1.root_potential(), self.h2.root_potential())
    }

    fn explore(&mut self, pos: Pos) {
        self.h1.explore(pos);
        self.h2.explore(pos);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MirrorHeuristic<H: Heuristic>(pub H);

pub struct MirrorHeuristicI<'a, H: Heuristic> {
    h: H::Instance<'a>,
}

impl<H: Heuristic> Heuristic for MirrorHeuristic<H>
where
    for<'a> H::Instance<'a>: HeuristicInstance<'a>,
    Pos: Copy + Eq + std::fmt::Debug + Default,
{
    type Instance<'a> = MirrorHeuristicI<'a, H>;

    fn name(&self) -> String {
        "mirror(".to_owned() + &self.0.name() + ")"
    }

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::Instance<'a> {
        MirrorHeuristicI {
            h: self.0.build(b, a),
        }
    }
}

impl<'a, H: Heuristic> HeuristicInstance<'a> for MirrorHeuristicI<'a, H>
where
    H::Instance<'a>: HeuristicInstance<'a>,
{
    fn h(&self, pos: Pos) -> Cost {
        self.h.h(pos.mirror())
    }

    type Hint = <<H as Heuristic>::Instance<'a> as HeuristicInstance<'a>>::Hint;

    fn is_seed_start_or_end(&self, pos: Pos) -> bool {
        self.h.is_seed_start_or_end(pos.mirror())
    }

    fn prune(&mut self, pos: Pos, hint: Self::Hint) -> (Cost, ()) {
        let c = self.h.prune(pos.mirror(), hint).0;
        (c, ())
    }

    fn h_with_hint(&self, pos: Pos, hint: Self::Hint) -> (Cost, Self::Hint) {
        self.h.h_with_hint(pos.mirror(), hint)
    }

    fn stats(&mut self) -> HeuristicStats {
        self.h.stats()
    }

    fn root_potential(&self) -> Cost {
        self.h.root_potential()
    }

    fn explore(&mut self, pos: Pos) {
        self.h.explore(pos.mirror())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PerfectHeuristic;

pub struct PerfectHeuristicI<'a> {
    a: Seq<'a>,
    b: Seq<'a>,
}

impl Heuristic for PerfectHeuristic {
    type Instance<'a> = PerfectHeuristicI<'a>;

    fn name(&self) -> String {
        "perfect".to_owned()
    }

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::Instance<'a> {
        PerfectHeuristicI { a, b }
    }
}

impl<'a> HeuristicInstance<'a> for PerfectHeuristicI<'a> {
    fn h(&self, Pos(i, j): Pos) -> Cost {
        bio::alignment::distance::simd::levenshtein(&self.a[i as usize..], &self.b[j as usize..])
            as _
    }
    type Hint = ();
}

#[derive(Debug, Clone, Copy)]
pub struct SymmetricHeuristic<H: Heuristic>(pub H);

type SymmetricHeuristicI<'a, H> = MaxHeuristicI<'a, H, MirrorHeuristic<H>>;

impl<H: Heuristic> Heuristic for SymmetricHeuristic<H>
where
    for<'a> H::Instance<'a>: HeuristicInstance<'a>,
    Pos: Copy + Eq + std::fmt::Debug + Default,
{
    type Instance<'a> = SymmetricHeuristicI<'a, H>;

    fn name(&self) -> String {
        "symm(".to_owned() + &self.0.name() + ")"
    }

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::Instance<'a> {
        let max_config = MaxHeuristic::<H, MirrorHeuristic<H>> {
            h1: self.0,
            h2: MirrorHeuristic(self.0),
        };
        max_config.build(a, b)
    }
}
