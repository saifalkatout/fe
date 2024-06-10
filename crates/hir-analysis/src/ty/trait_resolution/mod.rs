use std::collections::BTreeSet;

use hir::hir_def::IngotId;

use super::{
    canonical::{Canonical, Canonicalized, Solution},
    trait_def::TraitInstId,
    ty_def::{TyFlags, TyId},
};
use crate::{
    ty::{
        trait_resolution::{
            constraint::{collect_trait_constraints, ty_constraints},
            proof_forest::ProofForest,
        },
        unify::UnificationTable,
        visitor::collect_flags,
    },
    HirAnalysisDb,
};

pub(crate) mod constraint;
mod proof_forest;

#[salsa::tracked(return_ref)]
pub fn is_goal_satisfiable(
    db: &dyn HirAnalysisDb,
    assumptions: PredicateListId,
    goal: Canonical<TraitInstId>,
) -> GoalSatisfiability {
    let flags = collect_flags(db, goal.value);
    if flags.contains(TyFlags::HAS_INVALID) {
        return GoalSatisfiability::ContainsInvalid;
    };

    ProofForest::new(db, goal, assumptions).solve()
}

/// Checks if the given type is well-formed, i.e., the arguments of the given
/// type applications satisfies the constraints under the given assumptions.
#[salsa::tracked]
pub(crate) fn check_ty_wf(
    db: &dyn HirAnalysisDb,
    ty: TyId,
    assumptions: PredicateListId,
) -> WellFormedness {
    let (_, args) = ty.decompose_ty_app(db);

    for &arg in args {
        let wf = check_ty_wf(db, arg, assumptions);
        if !wf.is_wf() {
            return wf;
        }
    }

    let constraints = ty_constraints(db, ty);

    for &goal in constraints.list(db) {
        let mut table = UnificationTable::new(db);
        let canonical_goal = Canonicalized::new(db, goal);

        if let GoalSatisfiability::UnSat(subgoal) =
            is_goal_satisfiable(db, assumptions, canonical_goal.value)
        {
            let subgoal =
                subgoal.map(|subgoal| canonical_goal.extract_solution(&mut table, subgoal));
            return WellFormedness::IllFormed { goal, subgoal };
        }
    }

    WellFormedness::WellFormed
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WellFormedness {
    WellFormed,
    IllFormed {
        goal: TraitInstId,
        subgoal: Option<TraitInstId>,
    },
}

impl WellFormedness {
    fn is_wf(self) -> bool {
        matches!(self, WellFormedness::WellFormed)
    }
}

/// Checks if the given trait instance are well-formed, i.e., the arguments of
/// the trait satisfies all constraints under the given assumptions.
#[salsa::tracked]
pub(crate) fn check_trait_inst_wf(
    db: &dyn HirAnalysisDb,
    trait_inst: TraitInstId,
    assumptions: PredicateListId,
) -> WellFormedness {
    let constraints =
        collect_trait_constraints(db, trait_inst.def(db)).instantiate(db, trait_inst.args(db));

    for &goal in constraints.list(db) {
        let mut table = UnificationTable::new(db);
        let canonical_goal = Canonicalized::new(db, goal);
        if let GoalSatisfiability::UnSat(subgoal) =
            is_goal_satisfiable(db, assumptions, canonical_goal.value)
        {
            let subgoal =
                subgoal.map(|subgoal| canonical_goal.extract_solution(&mut table, subgoal));
            return WellFormedness::IllFormed { goal, subgoal };
        }
    }

    WellFormedness::WellFormed
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoalSatisfiability {
    /// Goal is satisfied with the unique solution.
    Satisfied(Solution<TraitInstId>),
    /// Goal is satisfied, but with multiple solutions.
    NeedsConfirmation(BTreeSet<Solution<TraitInstId>>),

    /// Goal contains invalid.
    ContainsInvalid,
    /// The gaol is not satisfied.
    /// It contains an unsatisfied subgoal if we can know the exact subgoal
    /// that makes the proof step stuck.
    UnSat(Option<Solution<TraitInstId>>),
}

impl GoalSatisfiability {
    pub fn is_satisfied(&self) -> bool {
        matches!(
            self,
            Self::Satisfied(_) | Self::NeedsConfirmation(_) | Self::ContainsInvalid
        )
    }
}

#[salsa::interned]
pub struct PredicateListId {
    #[return_ref]
    pub list: BTreeSet<TraitInstId>,
    pub ingot: IngotId,
}

impl PredicateListId {
    pub(super) fn merge(self, db: &dyn HirAnalysisDb, other: Self) -> Self {
        let mut predicates = self.list(db).clone();
        predicates.extend(other.list(db));
        PredicateListId::new(db, predicates, self.ingot(db))
    }

    pub(super) fn empty_list(db: &dyn HirAnalysisDb) -> Self {
        Self::new(db, BTreeSet::new(), IngotId::dummy())
    }

    fn extend_by_super(self, db: &dyn HirAnalysisDb) -> Self {
        let mut list = self.list(db).clone();
        for &pred in self.list(db) {
            for &super_trait in pred.def(db).super_traits(db).iter() {
                let super_trait = super_trait.instantiate(db, pred.args(db));
                list.insert(super_trait);
            }
        }

        Self::new(db, list, self.ingot(db))
    }
}