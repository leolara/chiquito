use core::fmt::Debug;
use std::{collections::HashMap, rc::Rc};

use halo2_proofs::{
    arithmetic::Field,
    plonk::{Advice, Column as Halo2Column},
};

use crate::{
    ast::{
        query::Queriable, Circuit as astCircuit, Expr, FixedSignal, ForwardSignal,
        ImportedHalo2Advice, ImportedHalo2Fixed, SharedSignal, StepType,
    },
    ir::{Circuit, Column, ColumnType, Poly, PolyExpr, PolyLookup},
    util::uuid,
    wit_gen::TraceGenerator,
};

use self::{
    cell_manager::{CellManager, Placement},
    step_selector::{StepSelector, StepSelectorBuilder},
};

pub mod cell_manager;
pub mod step_selector;

/// A trait that represents a fixed column generation context. It provides an interface for
/// assigning values to fixed columns in a circuit at the specified offset.
pub trait FixedGenContext<F> {
    /// Takes a `Queriable` object representing the fixed column (lhs) and the value (rhs) to be
    /// assigned.
    fn assign(&mut self, offset: usize, lhs: Queriable<F>, rhs: F);
}

#[derive(Debug)]
pub struct CompilationUnit<F> {
    pub placement: Placement,
    pub selector: StepSelector<F>,
    pub step_types: HashMap<u32, Rc<StepType<F>>>,
    pub forward_signals: Vec<ForwardSignal>,
    pub shared_signals: Vec<SharedSignal>,
    pub fixed_signals: Vec<FixedSignal>,

    pub annotations: HashMap<u32, String>,

    pub columns: Vec<Column>,
    pub exposed: Vec<(Column, i32)>,
    pub polys: Vec<Poly<F>>,
    pub lookups: Vec<PolyLookup<F>>,
}

impl<F> Default for CompilationUnit<F> {
    fn default() -> Self {
        Self {
            placement: Default::default(),
            selector: Default::default(),
            step_types: Default::default(),
            forward_signals: Default::default(),
            shared_signals: Default::default(),
            fixed_signals: Default::default(),

            annotations: Default::default(),

            columns: Default::default(),
            exposed: Default::default(),
            polys: Default::default(),
            lookups: Default::default(),
        }
    }
}

impl<F> CompilationUnit<F> {
    fn find_halo2_advice(&self, to_find: ImportedHalo2Advice) -> Option<Column> {
        for column in self.columns.iter() {
            if let Some(advice) = column.halo2_advice {
                if advice == to_find {
                    return Some(column.clone());
                }
            }
        }

        None
    }

    fn find_halo2_advice_native(&self, halo2_advice: Halo2Column<Advice>) -> Option<Column> {
        for column in self.columns.iter() {
            if let Some(advice) = column.halo2_advice {
                if advice.column == halo2_advice {
                    return Some(column.clone());
                }
            }
        }

        None
    }

    fn find_halo2_fixed(&self, to_find: ImportedHalo2Fixed) -> Option<Column> {
        for column in self.columns.iter() {
            if let Some(fixed) = column.halo2_fixed {
                if fixed == to_find {
                    return Some(column.clone());
                }
            }
        }

        None
    }
}

pub struct Compiler<CM: CellManager, SSB: StepSelectorBuilder> {
    cell_manager: CM,
    step_selector_builder: SSB,
}

impl<CM: CellManager, SSB: StepSelectorBuilder> Compiler<CM, SSB> {
    pub fn new(cell_manager: CM, step_selector_builder: SSB) -> Compiler<CM, SSB> {
        Compiler {
            cell_manager,
            step_selector_builder,
        }
    }

    pub fn compile<F: Field + Clone, TraceArgs>(
        &self,
        sc: &astCircuit<F, TraceArgs>,
    ) -> (Circuit<F>, Option<TraceGenerator<F, TraceArgs>>) {
        let mut unit = CompilationUnit::<F> {
            annotations: {
                let mut acc = sc.annotations.clone();
                for step in sc.step_types.values() {
                    acc.extend(step.annotations.clone());
                }

                acc
            },
            ..Default::default()
        };

        let halo2_advice_columns: Vec<Column> = sc
            .halo2_advice
            .iter()
            .map(|signal| {
                if let Some(annotation) = unit.annotations.get(&signal.uuid()) {
                    Column::new_halo2_advice(format!("halo2 advice {}", annotation), *signal)
                } else {
                    Column::new_halo2_advice("halo2 advice", *signal)
                }
            })
            .collect();

        let halo2_fixed_columns: Vec<Column> = sc
            .halo2_fixed
            .iter()
            .map(|signal| {
                if let Some(annotation) = unit.annotations.get(&signal.uuid()) {
                    Column::new_halo2_fixed(format!("halo2 fixed {}", annotation), *signal)
                } else {
                    Column::new_halo2_fixed("halo2 fixed", *signal)
                }
            })
            .collect();

        unit.columns = vec![halo2_advice_columns, halo2_fixed_columns].concat();
        unit.step_types = sc.step_types.clone();
        unit.forward_signals = sc.forward_signals.clone();
        unit.shared_signals = sc.shared_signals.clone();
        unit.fixed_signals = sc.fixed_signals.clone();

        self.cell_manager.place(&mut unit);

        if (!unit.shared_signals.is_empty() || !unit.fixed_signals.is_empty())
            && !unit.placement.same_height()
        {
            panic!("Shared signals and fixed signals are not supported for circuits with different step heights. Using a different cell manager might fix this problem.");
        }

        for forward_signal in sc.exposed.clone() {
            let forward_placement = unit.placement.get_forward_placement(&forward_signal);
            let exposed = (forward_placement.column, forward_placement.rotation);
            unit.exposed.push(exposed);
        }

        self.step_selector_builder.build::<F>(&mut unit);

        for step in unit.step_types.clone().values() {
            self.compile_step(&mut unit, step);
        }

        let q_enable = Column {
            annotation: "q_enable".to_owned(),
            ctype: ColumnType::Fixed,
            halo2_advice: None,
            halo2_fixed: None,
            phase: 0,
            id: uuid(),
        };

        unit.columns.push(q_enable.clone());

        self.add_q_enable(&mut unit, q_enable.clone());

        let q_first = if let Some(step_type) = sc.first_step {
            let q_first = Column {
                annotation: "q_first".to_owned(),
                ctype: ColumnType::Fixed,
                halo2_advice: None,
                halo2_fixed: None,
                phase: 0,
                id: uuid(),
            };
            unit.columns.push(q_first.clone());

            let step = unit.step_types.get(&step_type).expect("step not found");

            let poly = PolyExpr::Mul(vec![
                PolyExpr::<F>::Query(q_first.clone(), 0, "q_first".to_owned()),
                unit.selector.unselect(step.uuid()),
            ]);

            unit.polys.push(Poly {
                annotation: "q_first".to_string(),
                expr: poly,
            });

            Some(q_first)
        } else {
            None
        };

        let q_last = if let Some(step_type) = sc.last_step {
            let q_last = Column {
                annotation: "q_last".to_owned(),
                ctype: ColumnType::Fixed,
                halo2_advice: None,
                halo2_fixed: None,
                phase: 0,
                id: uuid(),
            };
            unit.columns.push(q_last.clone());

            let step = unit.step_types.get(&step_type).expect("step not found");

            let poly = PolyExpr::Mul(vec![
                PolyExpr::<F>::Query(q_last.clone(), 0, "q_last".to_owned()),
                unit.selector.unselect(step.uuid()),
            ]);

            unit.polys.push(Poly {
                annotation: "q_last".to_string(),
                expr: poly,
            });

            Some(q_last)
        } else {
            None
        };

        (
            Circuit::<F> {
                placement: unit.placement,
                selector: unit.selector,
                columns: unit.columns,
                exposed: unit.exposed,
                polys: unit.polys,
                lookups: unit.lookups,
                step_types: unit.step_types,
                q_enable,
                q_first,
                q_last,

                fixed_gen: sc.fixed_gen.as_ref().map(|v| Rc::clone(v)),
            },
            sc.trace.as_ref().map(|v| TraceGenerator::new(Rc::clone(v))),
        )
    }

    fn compile_step<F: Clone + Debug>(&self, unit: &mut CompilationUnit<F>, step: &StepType<F>) {
        let step_annotation = unit
            .annotations
            .get(&step.uuid())
            .unwrap_or(&"??".to_string())
            .to_owned();

        for constr in step.constraints.iter() {
            let constraint = self.transform_expr(unit, step, &constr.expr.clone());
            let poly = unit.selector.select(step.uuid(), &constraint);

            unit.polys.push(Poly {
                expr: poly,
                annotation: format!(
                    "{}::{} => {:?}",
                    step_annotation.clone(),
                    constr.annotation.clone(),
                    constr.expr
                ),
            })
        }

        // TODO only transition_constraints should have rotations
        for constr in step.transition_constraints.iter() {
            let constraint = self.transform_expr(unit, step, &constr.expr.clone());
            let poly = unit.selector.select(step.uuid(), &constraint);

            unit.polys.push(Poly {
                expr: poly,
                annotation: format!(
                    "{}::{} => {:?}",
                    step_annotation.clone(),
                    constr.annotation.clone(),
                    constr.expr
                ),
            })
        }

        for lookup in step.lookups.iter() {
            let poly_lookup = PolyLookup {
                annotation: lookup.annotation.clone(),
                exprs: lookup
                    .exprs
                    .iter()
                    .map(|(src, dest)| {
                        let src_poly = self.transform_expr(unit, step, &src.expr);
                        let dest_poly = self.transform_expr(unit, step, dest);
                        let src_selected = unit.selector.select(step.uuid(), &src_poly);

                        (src_selected, dest_poly)
                    })
                    .collect(),
            };

            unit.lookups.push(poly_lookup);
        }
    }

    fn place_queriable<F: Clone>(
        &self,
        unit: &CompilationUnit<F>,
        step: &StepType<F>,
        q: Queriable<F>,
    ) -> PolyExpr<F> {
        match q {
            Queriable::Internal(signal) => {
                let placement = unit
                    .placement
                    .find_internal_signal_placement(step.uuid(), &signal);

                let annotation = if let Some(annotation) = unit.annotations.get(&signal.uuid()) {
                    format!(
                        "{}[{}, {}]",
                        annotation, placement.column.annotation, placement.rotation
                    )
                } else {
                    format!("[{}, {}]", placement.column.annotation, placement.rotation)
                };

                PolyExpr::Query(placement.column, placement.rotation, annotation)
            }
            Queriable::Forward(forward, next) => {
                let placement = unit.placement.get_forward_placement(&forward);

                let super_rotation = placement.rotation
                    + if next {
                        unit.placement.step_height(step) as i32
                    } else {
                        0
                    };

                let annotation = if let Some(annotation) = unit.annotations.get(&forward.uuid()) {
                    if next {
                        format!(
                            "next({})[{}, {}]",
                            annotation, placement.column.annotation, super_rotation
                        )
                    } else {
                        format!(
                            "{}[{}, {}]",
                            annotation, placement.column.annotation, super_rotation
                        )
                    }
                } else {
                    format!("[{}, {}]", placement.column.annotation, super_rotation)
                };
                PolyExpr::Query(placement.column, super_rotation, annotation)
            }
            Queriable::Shared(shared, rot) => {
                let placement = unit.placement.get_shared_placement(&shared);

                let super_rotation =
                    placement.rotation + rot * (unit.placement.step_height(step) as i32);

                let annotation = if let Some(annotation) = unit.annotations.get(&shared.uuid()) {
                    if rot == 0 {
                        format!(
                            "{}[{}, {}]",
                            annotation, placement.column.annotation, super_rotation
                        )
                    } else {
                        format!(
                            "shared_rot_{}({})[{}, {}]",
                            rot, annotation, placement.column.annotation, super_rotation
                        )
                    }
                } else {
                    format!("[{}, {}]", placement.column.annotation, super_rotation)
                };
                PolyExpr::Query(placement.column, super_rotation, annotation)
            }
            Queriable::Fixed(fixed, rot) => {
                let placement = unit.placement.get_fixed_placement(&fixed);

                let super_rotation =
                    placement.rotation + rot * (unit.placement.step_height(step) as i32);

                let annotation = if let Some(annotation) = unit.annotations.get(&fixed.uuid()) {
                    if rot == 0 {
                        format!(
                            "{}[{}, {}]",
                            annotation, placement.column.annotation, super_rotation
                        )
                    } else {
                        format!(
                            "fixed_rot_{}({})[{}, {}]",
                            rot, annotation, placement.column.annotation, super_rotation
                        )
                    }
                } else {
                    format!("[{}, {}]", placement.column.annotation, super_rotation)
                };
                PolyExpr::Query(placement.column, super_rotation, annotation)
            }
            Queriable::StepTypeNext(step_type_handle) => {
                let super_rotation = unit.placement.step_height(step);
                let dest_step = unit
                    .step_types
                    .get(&step_type_handle.uuid())
                    .expect("step not found");

                unit.selector.next_expr(dest_step.uuid(), super_rotation)
            }
            Queriable::Halo2AdviceQuery(signal, rot) => {
                let annotation = if let Some(annotation) = unit.annotations.get(&signal.uuid()) {
                    format!("[{}, {}]", annotation, rot)
                } else {
                    format!("[halo2_advice?, {}]", rot)
                };
                PolyExpr::Query(
                    unit.find_halo2_advice(signal)
                        .expect("halo2 advice column not found"),
                    rot,
                    annotation,
                )
            }
            Queriable::Halo2FixedQuery(signal, rot) => {
                let annotation = if let Some(annotation) = unit.annotations.get(&signal.uuid()) {
                    format!("[{}, {}]", annotation, rot)
                } else {
                    format!("[halo2_fixed?, {}]", rot)
                };
                PolyExpr::Query(
                    unit.find_halo2_fixed(signal)
                        .expect("halo2 fixed column not found"),
                    rot,
                    annotation,
                )
            }
            Queriable::_unaccessible(_) => panic!("jarrl"),
        }
    }

    fn transform_expr<F: Clone>(
        &self,
        unit: &CompilationUnit<F>,
        step: &StepType<F>,
        source: &Expr<F>,
    ) -> PolyExpr<F> {
        match source.clone() {
            Expr::Const(c) => PolyExpr::Const(c),
            Expr::Sum(v) => PolyExpr::Sum(
                v.into_iter()
                    .map(|e| self.transform_expr(unit, step, &e))
                    .collect(),
            ),
            Expr::Mul(v) => PolyExpr::Mul(
                v.into_iter()
                    .map(|e| self.transform_expr(unit, step, &e))
                    .collect(),
            ),
            Expr::Neg(v) => PolyExpr::Neg(Box::new(self.transform_expr(unit, step, &v))),
            Expr::Pow(v, exp) => PolyExpr::Pow(Box::new(self.transform_expr(unit, step, &v)), exp),
            Expr::Query(q) => self.place_queriable(unit, step, q),
            Expr::Halo2Expr(expr) => PolyExpr::Halo2Expr(expr),
        }
    }

    fn add_q_enable<F: Clone>(&self, unit: &mut CompilationUnit<F>, q_enable: Column) {
        unit.polys = unit
            .polys
            .iter()
            .map(|poly| Poly {
                annotation: poly.annotation.clone(),
                expr: PolyExpr::Mul(vec![
                    PolyExpr::<F>::Query(q_enable.clone(), 0, "q_enable".to_owned()),
                    poly.expr.clone(),
                ]),
            })
            .collect();

        unit.lookups = unit
            .lookups
            .iter()
            .map(|lookup| PolyLookup {
                annotation: lookup.annotation.clone(),
                exprs: lookup
                    .exprs
                    .iter()
                    .map(|(src, dest)| {
                        (
                            PolyExpr::Mul(vec![
                                PolyExpr::<F>::Query(q_enable.clone(), 0, "q_enable".to_owned()),
                                src.clone(),
                            ]),
                            dest.clone(),
                        )
                    })
                    .collect(),
            })
            .collect();
    }
}
