use std::{fmt::Debug, hash::Hash};

use crate::{
    field::Field,
    poly::{simplify::simplify_mul, Expr},
};

use super::{ExprDecomp, SignalFactory};

pub fn reduce_degre<F: Field, V: Clone + Eq + PartialEq + Hash + Debug, SF: SignalFactory<V>>(
    expr: Expr<F, V>,
    max_degree: usize,
    signal_factory: &mut SF,
) -> ExprDecomp<F, V> {
    reduce_degree_recursive(expr, max_degree, max_degree, signal_factory)
}

fn reduce_degree_recursive<
    F: Field,
    V: Clone + Eq + PartialEq + Hash + Debug,
    SF: SignalFactory<V>,
>(
    expr: Expr<F, V>,
    total_max_degree: usize,
    partial_max_degree: usize,
    signal_factory: &mut SF,
) -> ExprDecomp<F, V> {
    if expr.degree() <= partial_max_degree {
        return ExprDecomp::from(expr);
    }

    match expr {
        Expr::Const(_) => ExprDecomp::from(expr),
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

            let root_expr = Expr::Sum(ses_reduction.iter().map(|r| r.root_expr.clone()).collect());

            ExprDecomp::merge(root_expr, ses_reduction)
        }
        Expr::Mul(ses) => {
            reduce_degree_mul(ses, total_max_degree, partial_max_degree, signal_factory)
        }
        Expr::Neg(se) => {
            let mut reduction =
                reduce_degree_recursive(*se, total_max_degree, partial_max_degree, signal_factory);
            reduction.root_expr = Expr::Neg(Box::new(reduction.root_expr));

            reduction
        }
        Expr::Pow(se, exp) => reduce_degree_mul(
            std::vec::from_elem(*se, exp as usize),
            total_max_degree,
            partial_max_degree,
            signal_factory,
        ),
        Expr::Query(_) => ExprDecomp::from(expr),
        Expr::Halo2Expr(_) => unimplemented!(),
    }
}

fn reduce_degree_mul<F: Field, V: Clone + Eq + PartialEq + Hash + Debug, SF: SignalFactory<V>>(
    ses: Vec<Expr<F, V>>,
    total_max_degree: usize,
    partial_max_degree: usize,
    signal_factory: &mut SF,
) -> ExprDecomp<F, V> {
    if partial_max_degree == 1 {
        let reduction = reduce_degree_mul(ses, total_max_degree, total_max_degree, signal_factory);
        let signal = signal_factory.create("virtual signal");

        return ExprDecomp::inherit(Expr::Query(signal.clone()), signal, reduction);
    }

    let ses = simplify_mul(ses);

    let mut first = true;

    let ses_reduced: Vec<ExprDecomp<F, V>> = ses
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

    let mut for_root: Vec<ExprDecomp<F, V>> = Default::default();
    let mut to_simplify: Vec<ExprDecomp<F, V>> = Default::default();

    let mut current_degree = 0;
    for se in ses_reduced {
        if se.root_expr.degree() + current_degree < partial_max_degree {
            current_degree += se.root_expr.degree();
            for_root.push(se);
        } else {
            to_simplify.push(se);
        }
    }

    assert!(!for_root.is_empty());
    assert!(!to_simplify.is_empty());

    let rest_signal = signal_factory.create("rest expr");
    let mut root_exprs: Vec<_> = for_root.iter().map(|se| se.root_expr.clone()).collect();
    root_exprs.push(Expr::Query(rest_signal.clone()));
    let root_expr = Expr::Mul(root_exprs);

    let simplified = reduce_degree_recursive(
        Expr::Mul(to_simplify.iter().map(|se| se.root_expr.clone()).collect()),
        total_max_degree,
        total_max_degree,
        signal_factory,
    );

    let mut result = ExprDecomp::merge(
        root_expr,
        [for_root, to_simplify.clone(), vec![simplified.clone()]]
            .into_iter()
            .flatten()
            .collect(),
    );

    result.auto_eq(rest_signal, simplified.root_expr);

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

        assert_eq!(format!("{:#?}", result.root_expr), "(a * v1)");
        assert_eq!(format!("{:#?}", result.exprs[0]), "((b * c) + (-v1))");
        assert_eq!(result.exprs.len(), 1);
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

        assert_eq!(format!("{:#?}", result.root_expr), "((a + b) * v1)");
        assert_eq!(
            format!("{:#?}", result.exprs[0]),
            "(((b + c) * (a + c)) + (-v1))"
        );
        assert_eq!(result.exprs.len(), 1);
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

        assert_eq!(format!("{:#?}", result.root_expr), "(a * v1)");
        assert_eq!(format!("{:#?}", result.exprs[0]), "((d * e) + (-v3))");
        assert_eq!(format!("{:#?}", result.exprs[1]), "((c * v3) + (-v2))");
        assert_eq!(format!("{:#?}", result.exprs[2]), "((b * v2) + (-v1))");
        assert_eq!(result.exprs.len(), 3);
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

        assert_eq!(format!("{:#?}", result.root_expr), "(0x1 + (-(a * v1)))");
        assert_eq!(format!("{:#?}", result.exprs[0]), "((d * e) + (-v3))");
        assert_eq!(format!("{:#?}", result.exprs[1]), "((c * v3) + (-v2))");
        assert_eq!(format!("{:#?}", result.exprs[2]), "((b * v2) + (-v1))");
        assert_eq!(result.exprs.len(), 3);
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
            format!("{:#?}", result.root_expr),
            "((a * v1) + (-(b * v3)))"
        );
        assert_eq!(format!("{:#?}", result.exprs[0]), "((a * a) + (-v2))");
        assert_eq!(format!("{:#?}", result.exprs[1]), "((a * v2) + (-v1))");
        assert_eq!(format!("{:#?}", result.exprs[2]), "((d * e) + (-v4))");
        assert_eq!(format!("{:#?}", result.exprs[3]), "((c * v4) + (-v3))");
        assert_eq!(result.exprs.len(), 4);
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