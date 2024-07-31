// Pub cse function receives a circuit and returns a new circuit with common subexpressions replaced
// by signals. Pub cse gets the

use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    marker::PhantomData,
};

use crate::{
    field::Field,
    poly::{
        self,
        cse::{create_common_ses_signal, replace_expr},
        Expr, HashResult, VarAssignments,
    },
    sbpir::{query::Queriable, InternalSignal, SBPIR},
    wit_gen::NullTraceGenerator,
};


/// Common Subexpression Elimination (CSE) optimization.
/// This optimization replaces common subexpressions with new internal signals for the step type.
/// This is done by each time finding the optimal subexpression to replace and creating a new signal
/// for it and replacing it in all constraints.
/// The process is repeated until no more common subexpressions are found.
/// Equivalent expressions are found by hashing the expressions with random assignments to the queriables. Using
/// the Schwartz-Zippel lemma, we can determine if two expressions are equivalent with high probability.
pub(super) fn cse<F: Field + Hash>(
    mut circuit: SBPIR<F, NullTraceGenerator>,
) -> SBPIR<F, NullTraceGenerator> {
    for (_, step_type) in circuit.step_types.iter_mut() {
        let mut signal_factory = SignalFactory::default();
        let mut replaced_hashes = HashSet::new();

        loop {
            let mut queriables = Vec::<Queriable<F>>::new();

            circuit.forward_signals.iter().for_each(|signal| {
                queriables.push(Queriable::Forward(signal.clone(), false));
                queriables.push(Queriable::Forward(signal.clone(), true));
            });
            step_type.signals.iter().for_each(|signal| {
                queriables.push(Queriable::Internal(signal.clone()));
            });

            // Generate random assignments for the queriables
            let mut rng = ChaCha20Rng::seed_from_u64(0);
            let random_assignments: VarAssignments<F, Queriable<F>> = queriables
                .iter()
                .cloned()
                .map(|q| (q, F::random(&mut rng)))
                .collect();

            // Turn all Expr<F, V, ()> into Expr<F, V, HashResult>
            let mut step_type_with_hash = step_type.with_meta(|expr| {
                let hashed_expr = expr.hash(&random_assignments);
                hashed_expr.meta().clone()
            });

            // Extract all the expressions from the step type
            let mut exprs = Vec::<Expr<F, Queriable<F>, HashResult>>::new();

            for constraint in &step_type_with_hash.constraints {
                exprs.push(constraint.expr.clone());
            }

            // Find the optimal subexpression to replace
            if let Some(common_expr) = find_optimal_subexpression(&exprs, &replaced_hashes) {
                // Add the hash of the replaced expression to the set
                replaced_hashes.insert(common_expr.meta().hash);
                // Create a new signal for the common subexpression
                let (common_se, decomp) =
                    create_common_ses_signal(&common_expr, &mut signal_factory);

                // Add the new signal to the step type and a constraint for it
                decomp.auto_signals.iter().for_each(|(q, expr)| {
                    if let Queriable::Internal(signal) = q {
                        step_type_with_hash.add_internal(signal.clone());
                    }
                    step_type_with_hash
                        .auto_signals
                        .insert(q.clone(), expr.clone());
                    step_type_with_hash.add_constr(format!("{:?}", q), expr.clone());
                });

                // Replace the common subexpression in all constraints
                step_type_with_hash.decomp_constraints(|expr| {
                    replace_expr(expr, &common_se, &mut signal_factory, decomp.clone())
                });
            } else {
                // No more common subexpressions found, exit the loop
                break;
            }
            *step_type = step_type_with_hash.without_meta();
        }
    }
    circuit
}

#[derive(Debug, Clone, Copy)]
struct SubexprInfo {
    count: usize,
    degree: usize,
}

impl SubexprInfo {
    fn new(count: usize, degree: usize) -> Self {
        Self { count, degree }
    }

    fn update(&mut self, degree: usize) {
        self.count += 1;
        self.degree = self.degree.max(degree);
    }
}

fn find_optimal_subexpression<F: Field + Hash>(
    exprs: &Vec<Expr<F, Queriable<F>, HashResult>>,
    replaced_hashes: &HashSet<u64>,
) -> Option<Expr<F, Queriable<F>, HashResult>> {
    let mut count_map = HashMap::<u64, SubexprInfo>::new();
    let mut hash_to_expr = HashMap::<u64, Expr<F, Queriable<F>, HashResult>>::new();

    // Extract all subexpressions and count them
    for expr in exprs.iter() {
        count_subexpressions(expr, &mut count_map, &mut hash_to_expr, replaced_hashes);
    }

    // Find the best common subexpression to replace
    let common_ses = count_map
        .into_iter()
        .filter(|&(hash, info)| {
            info.count > 1 && info.degree > 1 && !replaced_hashes.contains(&hash)
        })
        .collect::<HashMap<_, _>>();

    let best_subexpr = common_ses
        .iter()
        .max_by_key(|&(_, info)| (info.degree, info.count))
        .map(|(&hash, info)| (hash, info.count, info.degree));

    if let Some((hash, _count, _degree)) = best_subexpr {
        hash_to_expr.get(&hash).cloned()
    } else {
        None
    }
}

fn count_subexpressions<F: Field + Hash>(
    expr: &Expr<F, Queriable<F>, HashResult>,
    count_map: &mut HashMap<u64, SubexprInfo>,
    hash_to_expr: &mut HashMap<u64, Expr<F, Queriable<F>, HashResult>>,
    replaced_hashes: &HashSet<u64>,
) {
    let degree = expr.degree();
    let hash_result = expr.meta().hash;

    // Only count and store if not already replaced
    if !replaced_hashes.contains(&hash_result) {
        // Store the expression with its hash
        hash_to_expr.insert(hash_result, expr.clone());

        count_map
            .entry(hash_result)
            .and_modify(|info| info.update(degree))
            .or_insert(SubexprInfo::new(1, degree));
    }

    // Recurse into subexpressions
    match expr {
        Expr::Const(_, _) | Expr::Query(_, _) => {}
        Expr::Sum(exprs, _) | Expr::Mul(exprs, _) => {
            for subexpr in exprs {
                count_subexpressions(subexpr, count_map, hash_to_expr, replaced_hashes);
            }
        }
        Expr::Neg(subexpr, _) | Expr::MI(subexpr, _) => {
            count_subexpressions(subexpr, count_map, hash_to_expr, replaced_hashes);
        }
        Expr::Pow(subexpr, _, _) => {
            count_subexpressions(subexpr, count_map, hash_to_expr, replaced_hashes);
        }
        _ => {}
    }
}

// Basic signal factory.
#[derive(Default)]
struct SignalFactory<F> {
    count: u64,
    _p: PhantomData<F>,
}

impl<F> poly::SignalFactory<Queriable<F>> for SignalFactory<F> {
    fn create<S: Into<String>>(&mut self, annotation: S) -> Queriable<F> {
        self.count += 1;
        Queriable::Internal(InternalSignal::new(format!(
            "{}-{}",
            annotation.into(),
            self.count
        )))
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use halo2_proofs::halo2curves::bn256::Fr;
    use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};

    use crate::{
        compiler::cse::cse,
        field::Field,
        poly::{Expr, ToExpr, VarAssignments},
        sbpir::{query::Queriable, InternalSignal, StepType, SBPIR},
        util::uuid,
        wit_gen::NullTraceGenerator,
    };

    use super::find_optimal_subexpression;

    #[test]
    fn test_find_optimal_subexpression() {
        let a: Queriable<Fr> = Queriable::Internal(InternalSignal::new("a"));
        let b = Queriable::Internal(InternalSignal::new("b"));
        let c = Queriable::Internal(InternalSignal::new("c"));
        let d = Queriable::Internal(InternalSignal::new("d"));
        let e = Queriable::Internal(InternalSignal::new("e"));
        let f = Queriable::Internal(InternalSignal::new("f"));
        let vars = vec![
            a.clone(),
            b.clone(),
            c.clone(),
            d.clone(),
            e.clone(),
            f.clone(),
        ];

        let expr1 = a * b + c;
        let expr2 = c + b + a;
        let expr3 = 4.expr() + a * b + c;
        let expr4 = e * f * d;
        let expr5 = expr1.clone() + expr4.clone();
        let exprs = vec![expr1, expr2, expr3, expr4, expr5];

        let mut rng = ChaCha20Rng::seed_from_u64(0);
        let mut rand_assignments = VarAssignments::new();
        for var in vars.iter() {
            rand_assignments.insert(var.clone(), Fr::random(&mut rng));
        }

        let mut hashed_exprs = Vec::new();
        for expr in exprs.iter() {
            let hashed_expr = expr.hash(&rand_assignments);
            hashed_exprs.push(hashed_expr);
        }

        find_optimal_subexpression(&hashed_exprs, &HashSet::new());
    }

    #[test]
    fn test_cse() {
        let a = InternalSignal::new("a");
        let b = InternalSignal::new("b");
        let c = InternalSignal::new("c");
        let d = InternalSignal::new("d");
        let e = InternalSignal::new("e");
        let f = InternalSignal::new("f");

        let expr1: Expr<Fr, Queriable<Fr>, ()> = Expr::Query(Queriable::Internal(a), ())
            * Expr::Query(Queriable::Internal(b), ())
            + Expr::Query(Queriable::Internal(c), ());
        let expr2 = Expr::Query(Queriable::Internal(c), ())
            + Expr::Query(Queriable::Internal(b), ())
            + Expr::Query(Queriable::Internal(a), ());
        let expr3: Expr<Fr, Queriable<Fr>, ()> = Expr::Const(Fr::from(4), ())
            + Expr::Query(Queriable::Internal(a), ()) * Expr::Query(Queriable::Internal(b), ())
            + Expr::Query(Queriable::Internal(c), ());
        let expr4 = Expr::Query(Queriable::Internal(e), ())
            * Expr::Query(Queriable::Internal(f), ())
            * Expr::Query(Queriable::Internal(d), ());
        let expr5 = expr1.clone() + expr4.clone();

        let mut step: StepType<Fr, ()> = StepType::new(uuid(), "example_step".to_string());

        step.add_internal(a);
        step.add_internal(b);
        step.add_internal(c);
        step.add_internal(d);
        step.add_internal(e);
        step.add_internal(f);

        step.add_constr("expr1".into(), expr1);
        step.add_constr("expr2".into(), expr2);
        step.add_constr("expr3".into(), expr3);
        step.add_constr("expr4".into(), expr4);
        step.add_constr("expr5".into(), expr5);

        let mut circuit: SBPIR<Fr, NullTraceGenerator> = SBPIR::default();
        circuit.add_step_type_def(step);

        println!("Circuit before CSE: {:#?}", circuit);

        let circuit = cse(circuit);

        println!("Circuit after CSE: {:#?}", circuit);
    }

    #[derive(Clone)]
    struct TestStruct {
        value: String,
    }

    #[test]
    fn test_step_type_with_meta() {
        let a = InternalSignal::new("a");
        let b = InternalSignal::new("b");
        let c = InternalSignal::new("c");
        let d = InternalSignal::new("d");
        let e = InternalSignal::new("e");
        let f = InternalSignal::new("f");

        let expr1: Expr<Fr, Queriable<Fr>, ()> = Expr::Query(Queriable::Internal(a), ())
            * Expr::Query(Queriable::Internal(b), ())
            + Expr::Query(Queriable::Internal(c), ());
        let expr2 = Expr::Query(Queriable::Internal(c), ())
            + Expr::Query(Queriable::Internal(b), ())
            + Expr::Query(Queriable::Internal(a), ());
        let expr3: Expr<Fr, Queriable<Fr>, ()> = Expr::Const(Fr::from(4), ())
            + Expr::Query(Queriable::Internal(a), ()) * Expr::Query(Queriable::Internal(b), ())
            + Expr::Query(Queriable::Internal(c), ());
        let expr4 = Expr::Query(Queriable::Internal(e), ())
            * Expr::Query(Queriable::Internal(f), ())
            * Expr::Query(Queriable::Internal(d), ());
        let expr5 = expr1.clone() + expr4.clone();

        let mut step: StepType<Fr, ()> = StepType::new(uuid(), "example_step".to_string());

        step.add_internal(a);
        step.add_internal(b);
        step.add_internal(c);
        step.add_internal(d);
        step.add_internal(e);
        step.add_internal(f);

        step.add_constr("expr1".into(), expr1);
        step.add_constr("expr2".into(), expr2);
        step.add_constr("expr3".into(), expr3);
        step.add_constr("expr4".into(), expr4);
        step.add_constr("expr5".into(), expr5);

        let step_with_meta = step.with_meta(|expr| TestStruct {
            value: format!("{:?}", expr),
        });

        for constraint in &step_with_meta.constraints {
            println!("{:?}", constraint.expr.meta().value);
        }
    }
}
