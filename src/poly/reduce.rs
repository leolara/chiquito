use std::{fmt::Debug, hash::Hash};

use crate::{
    field::Field,
    poly::{simplify::simplify_mul, Expr},
};

use super::{ConstrDecomp, SignalFactory};

pub fn reduce_degre<F: Field, V: Clone + Eq + PartialEq + Hash + Debug, SF: SignalFactory<V>>(
    constr: Expr<F, V>,
    max_degree: usize,
    signal_factory: &mut SF,
) -> ConstrDecomp<F, V> {
    reduce_degree_recursive(constr, max_degree, max_degree, signal_factory)
}

fn reduce_degree_recursive<
    F: Field,
    V: Clone + Eq + PartialEq + Hash + Debug,
    SF: SignalFactory<V>,
>(
    constr: Expr<F, V>,
    total_max_degree: usize,
    partial_max_degree: usize,
    signal_factory: &mut SF,
) -> ConstrDecomp<F, V> {
    if constr.degree() <= partial_max_degree {
        return ConstrDecomp::from(constr);
    }

    match constr {
        Expr::Const(_) => ConstrDecomp::from(constr),
        Expr::Sum(ses) => {
            let ses_reduction: Vec<_> = ses
                .iter()
                .map(|se| {
                    reduce_degree_recursive(
                        se.clone(),
                        total_max_degree,
                        partial_max_degree,
                        signal_factory,
                    )
                })
                .collect();

            let root_expr = Expr::Sum(
                ses_reduction
                    .iter()
                    .map(|r| r.root_constr.clone())
                    .collect(),
            );

            ConstrDecomp::merge(root_expr, ses_reduction)
        }
        Expr::Mul(ses) => {
            reduce_degree_mul(ses, total_max_degree, partial_max_degree, signal_factory)
        }
        Expr::Neg(se) => {
            let mut reduction =
                reduce_degree_recursive(*se, total_max_degree, partial_max_degree, signal_factory);
            reduction.root_constr = Expr::Neg(Box::new(reduction.root_constr));

            reduction
        }
        // TODO: decompose in Pow expressions instead of Mul
        Expr::Pow(se, exp) => reduce_degree_mul(
            std::vec::from_elem(*se, exp as usize),
            total_max_degree,
            partial_max_degree,
            signal_factory,
        ),
        Expr::Query(_) => ConstrDecomp::from(constr),
        Expr::Halo2Expr(_) => unimplemented!(),
    }
}

fn reduce_degree_mul<F: Field, V: Clone + Eq + PartialEq + Hash + Debug, SF: SignalFactory<V>>(
    ses: Vec<Expr<F, V>>,
    total_max_degree: usize,
    partial_max_degree: usize,
    signal_factory: &mut SF,
) -> ConstrDecomp<F, V> {
    // base case, if partial_max_degree == 1, the root expresion can only be a variable
    if partial_max_degree == 1 {
        let reduction = reduce_degree_mul(ses, total_max_degree, total_max_degree, signal_factory);
        let signal = signal_factory.create("virtual signal");

        return ConstrDecomp::inherit(Expr::Query(signal.clone()), signal, reduction);
    }

    let ses = simplify_mul(ses);

    // to reduce the problem for recursion, at least one expression should have lower degree than
    // total_max_degree
    let mut first = true;
    let ses_reduced: Vec<ConstrDecomp<F, V>> = ses
        .into_iter()
        .map(|se| {
            let partial_max_degree = if first {
                total_max_degree - 1
            } else {
                total_max_degree
            };
            let reduction =
                reduce_degree_recursive(se, total_max_degree, partial_max_degree, signal_factory);
            first = false;

            reduction
        })
        .collect();

    // for_root will be multipliers that will be included in the root expression
    let mut for_root: Vec<ConstrDecomp<F, V>> = Default::default();
    // to_simplify will be multipliers that will be recursively decomposed and subsituted by a
    // virtual signal in the root expression
    let mut to_simplify: Vec<ConstrDecomp<F, V>> = Default::default();

    let mut current_degree = 0;
    for se in ses_reduced {
        if se.root_constr.degree() + current_degree < partial_max_degree {
            current_degree += se.root_constr.degree();
            for_root.push(se);
        } else {
            to_simplify.push(se);
        }
    }

    assert!(!for_root.is_empty());
    assert!(!to_simplify.is_empty());

    let rest_signal = signal_factory.create("rest expr");
    let mut root_exprs: Vec<_> = for_root.iter().map(|se| se.root_constr.clone()).collect();
    root_exprs.push(Expr::Query(rest_signal.clone()));
    let root_expr = Expr::Mul(root_exprs);

    // recursion, for the part that exceeds the degree and will be substituted by a virtual signal
    let simplified = reduce_degree_recursive(
        Expr::Mul(
            to_simplify
                .iter()
                .map(|se| se.root_constr.clone())
                .collect(),
        ),
        total_max_degree,
        total_max_degree,
        signal_factory,
    );

    let mut result = ConstrDecomp::merge(
        root_expr,
        [for_root, to_simplify.clone(), vec![simplified.clone()]]
            .into_iter()
            .flatten()
            .collect(),
    );

    result.auto_eq(rest_signal, simplified.root_constr);

    result
}

#[cfg(test)]
mod test {
    use halo2curves::bn256::Fr;

    use crate::{
        ast::{query::Queriable, InternalSignal},
        poly::{Expr::*, ToExpr},
    };

    use super::{reduce_degre, reduce_degree_mul, SignalFactory};

    #[derive(Default)]
    struct TestSignalFactory {
        counter: u32,
    }

    impl SignalFactory<Queriable<Fr>> for TestSignalFactory {
        fn create<S: Into<String>>(&mut self, _annotation: S) -> Queriable<Fr> {
            self.counter += 1;

            Queriable::Internal(InternalSignal::new(format!("v{}", self.counter)))
        }
    }

    #[test]
    fn test_reduce_degree_mul() {
        let a: Queriable<Fr> = Queriable::Internal(InternalSignal::new("a"));
        let b: Queriable<Fr> = Queriable::Internal(InternalSignal::new("b"));
        let c: Queriable<Fr> = Queriable::Internal(InternalSignal::new("c"));

        let result = reduce_degree_mul(
            vec![a.expr(), b.expr(), c.expr()],
            2,
            2,
            &mut TestSignalFactory::default(),
        );

        assert_eq!(format!("{:#?}", result.root_constr), "(a * v1)");
        assert_eq!(format!("{:#?}", result.constrs[0]), "((b * c) + (-v1))");
        assert_eq!(result.constrs.len(), 1);
        assert!(result
            .auto_signals
            .iter()
            .any(|(s, expr)| format!("{:#?}: {:#?}", s, expr) == "v1: (b * c)"));
        assert_eq!(result.auto_signals.len(), 1);

        let result = reduce_degree_mul(
            vec![(a + b), (b + c), (a + c)],
            2,
            2,
            &mut TestSignalFactory::default(),
        );

        assert_eq!(format!("{:#?}", result.root_constr), "((a + b) * v1)");
        assert_eq!(
            format!("{:#?}", result.constrs[0]),
            "(((b + c) * (a + c)) + (-v1))"
        );
        assert_eq!(result.constrs.len(), 1);
        assert!(result
            .auto_signals
            .iter()
            .any(|(s, expr)| format!("{:#?}: {:#?}", s, expr) == "v1: ((b + c) * (a + c))"));
        assert_eq!(result.auto_signals.len(), 1);
    }

    #[test]
    fn test_reduce_degree() {
        let a: Queriable<Fr> = Queriable::Internal(InternalSignal::new("a"));
        let b: Queriable<Fr> = Queriable::Internal(InternalSignal::new("b"));
        let c: Queriable<Fr> = Queriable::Internal(InternalSignal::new("c"));
        let d: Queriable<Fr> = Queriable::Internal(InternalSignal::new("d"));
        let e: Queriable<Fr> = Queriable::Internal(InternalSignal::new("e"));

        let result = reduce_degre(a * b * c * d * e, 2, &mut TestSignalFactory::default());

        assert_eq!(format!("{:#?}", result.root_constr), "(a * v1)");
        assert_eq!(format!("{:#?}", result.constrs[0]), "((d * e) + (-v3))");
        assert_eq!(format!("{:#?}", result.constrs[1]), "((c * v3) + (-v2))");
        assert_eq!(format!("{:#?}", result.constrs[2]), "((b * v2) + (-v1))");
        assert_eq!(result.constrs.len(), 3);
        assert!(result
            .auto_signals
            .iter()
            .any(|(s, expr)| format!("{:#?}: {:#?}", s, expr) == "v2: (c * v3)"));
        assert!(result
            .auto_signals
            .iter()
            .any(|(s, expr)| format!("{:#?}: {:#?}", s, expr) == "v1: (b * v2)"));
        assert!(result
            .auto_signals
            .iter()
            .any(|(s, expr)| format!("{:#?}: {:#?}", s, expr) == "v3: (d * e)"));
        assert_eq!(result.auto_signals.len(), 3);

        let result = reduce_degre(
            1.expr() - (a * b * c * d * e),
            2,
            &mut TestSignalFactory::default(),
        );

        assert_eq!(format!("{:#?}", result.root_constr), "(0x1 + (-(a * v1)))");
        assert_eq!(format!("{:#?}", result.constrs[0]), "((d * e) + (-v3))");
        assert_eq!(format!("{:#?}", result.constrs[1]), "((c * v3) + (-v2))");
        assert_eq!(format!("{:#?}", result.constrs[2]), "((b * v2) + (-v1))");
        assert_eq!(result.constrs.len(), 3);
        assert!(result
            .auto_signals
            .iter()
            .any(|(s, expr)| format!("{:#?}: {:#?}", s, expr) == "v2: (c * v3)"));
        assert!(result
            .auto_signals
            .iter()
            .any(|(s, expr)| format!("{:#?}: {:#?}", s, expr) == "v1: (b * v2)"));
        assert!(result
            .auto_signals
            .iter()
            .any(|(s, expr)| format!("{:#?}: {:#?}", s, expr) == "v3: (d * e)"));
        assert_eq!(result.auto_signals.len(), 3);

        let result = reduce_degre(
            Pow(Box::new(a.expr()), 4) - (b * c * d * e),
            2,
            &mut TestSignalFactory::default(),
        );

        assert_eq!(
            format!("{:#?}", result.root_constr),
            "((a * v1) + (-(b * v3)))"
        );
        assert_eq!(format!("{:#?}", result.constrs[0]), "((a * a) + (-v2))");
        assert_eq!(format!("{:#?}", result.constrs[1]), "((a * v2) + (-v1))");
        assert_eq!(format!("{:#?}", result.constrs[2]), "((d * e) + (-v4))");
        assert_eq!(format!("{:#?}", result.constrs[3]), "((c * v4) + (-v3))");
        assert_eq!(result.constrs.len(), 4);
        assert!(result
            .auto_signals
            .iter()
            .any(|(s, expr)| format!("{:#?}: {:#?}", s, expr) == "v2: (a * a)"));
        assert!(result
            .auto_signals
            .iter()
            .any(|(s, expr)| format!("{:#?}: {:#?}", s, expr) == "v1: (a * v2)"));
        assert!(result
            .auto_signals
            .iter()
            .any(|(s, expr)| format!("{:#?}: {:#?}", s, expr) == "v4: (d * e)"));
        assert!(result
            .auto_signals
            .iter()
            .any(|(s, expr)| format!("{:#?}: {:#?}", s, expr) == "v3: (c * v4)"));
        assert_eq!(result.auto_signals.len(), 4);
    }
}
