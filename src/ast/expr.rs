use std::ops::{Add, BitOr, Mul, Neg, Sub};

use halo2_proofs::halo2curves::FieldExt;

use self::query::Queriable;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr<F> {
    Const(F),
    Sum(Vec<Expr<F>>),
    Mul(Vec<Expr<F>>),
    Neg(Box<Expr<F>>),
    Pow(Box<Expr<F>>, u32),
    Query(Queriable<F>),
    Equal(Box<Expr<F>>, Box<Expr<F>>),
}

pub trait ToExpr<F> {
    fn expr(&self) -> Expr<F>;
}

impl<F: Clone> ToExpr<F> for Expr<F> {
    fn expr(&self) -> Expr<F> {
        self.clone()
    }
}

pub trait ToField<F: FieldExt> {
    fn field(&self) -> F;
}

impl<F, RHS: Into<Expr<F>>> Add<RHS> for Expr<F> {
    type Output = Self;
    fn add(self, rhs: RHS) -> Self {
        use Expr::*;
        match self {
            Sum(mut xs) => {
                xs.push(rhs.into());
                Sum(xs)
            }
            e => Sum(vec![e, rhs.into()]),
        }
    }
}

impl<F, RHS: Into<Expr<F>>> Sub<RHS> for Expr<F> {
    type Output = Self;
    fn sub(self, rhs: RHS) -> Self {
        use Expr::*;
        match self {
            Sum(mut xs) => {
                xs.push(rhs.into().neg());
                Sum(xs)
            }
            e => Sum(vec![e, rhs.into().neg()]),
        }
    }
}

impl<F, RHS: Into<Expr<F>>> Mul<RHS> for Expr<F> {
    type Output = Self;
    fn mul(self, rhs: RHS) -> Self {
        use Expr::*;
        match self {
            Mul(mut xs) => {
                xs.push(rhs.into());
                Mul(xs)
            }
            e => Mul(vec![e, rhs.into()]),
        }
    }
}

impl<F> Neg for Expr<F> {
    type Output = Self;
    fn neg(self) -> Self {
        match self {
            Expr::Neg(xs) => *xs,
            e => Expr::Neg(Box::new(e)),
        }
    }
}

impl<F, RHS: Into<Expr<F>>> BitOr<RHS> for Expr<F> {
    type Output = Self;
    fn bitor(self, rhs: RHS) -> Self::Output {
        Expr::Equal(Box::new(self), Box::new(rhs.into()))
    }
}

macro_rules! impl_expr_like {
    ($type:ty) => {
        impl<F: halo2_proofs::arithmetic::FieldExt> From<$type> for Expr<F> {
            #[inline]
            fn from(value: $type) -> Self {
                Expr::Const(F::from(value as u64))
            }
        }

        impl<F: halo2_proofs::arithmetic::FieldExt> $crate::ast::ToExpr<F> for $type {
            #[inline]
            fn expr(&self) -> Expr<F> {
                Expr::Const(F::from(*self as u64))
            }
        }

        impl<F: halo2_proofs::arithmetic::FieldExt> $crate::ast::ToField<F> for $type {
            #[inline]
            fn field(&self) -> F {
                F::from(*self as u64)
            }
        }
    };
}

impl_expr_like!(bool);
impl_expr_like!(u8);
impl_expr_like!(u64);
impl_expr_like!(usize);

impl<F: FieldExt> From<i32> for Expr<F> {
    #[inline]
    fn from(value: i32) -> Self {
        Expr::Const(
            F::from(value.unsigned_abs() as u64)
                * if value.is_negative() {
                    -F::one()
                } else {
                    F::one()
                },
        )
    }
}

impl<F: FieldExt> ToExpr<F> for i32 {
    #[inline]
    fn expr(&self) -> Expr<F> {
        Expr::Const(
            F::from(self.unsigned_abs() as u64)
                * if self.is_negative() {
                    -F::one()
                } else {
                    F::one()
                },
        )
    }
}

impl<F: FieldExt> ToField<F> for i32 {
    #[inline]
    fn field(&self) -> F {
        F::from(self.unsigned_abs() as u64)
            * if self.is_negative() {
                -F::one()
            } else {
                F::one()
            }
    }
}

pub mod query {
    use std::{
        marker::PhantomData,
        ops::{Add, BitOr, Mul, Neg, Sub},
    };

    use crate::{
        ast::{ForwardSignal, InternalSignal},
        dsl::StepTypeHandler,
    };

    use super::{Expr, ToExpr};

    // Queriable
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Queriable<F> {
        Internal(InternalSignal),
        Forward(ForwardSignal),
        ForwardNext(ForwardSignal),
        StepTypeNext(StepTypeHandler), // uuid of step type
        _unaccessible(PhantomData<F>),
    }

    impl<F> Queriable<F> {
        pub fn next(&self) -> Queriable<F> {
            use Queriable::*;
            match self {
                Forward(s) => ForwardNext(*s),
                _ => panic!("can only next a queriable"),
            }
        }
    }

    impl<F> From<Queriable<F>> for Expr<F> {
        fn from(value: Queriable<F>) -> Self {
            Expr::Query(value)
        }
    }

    impl<F: Clone> ToExpr<F> for Queriable<F> {
        fn expr(&self) -> Expr<F> {
            Expr::Query((*self).clone())
        }
    }

    impl<F: Clone, RHS: Into<Expr<F>>> Add<RHS> for Queriable<F> {
        type Output = Expr<F>;

        fn add(self, rhs: RHS) -> Self::Output {
            self.expr() + rhs
        }
    }

    impl<F: Clone, RHS: Into<Expr<F>>> Sub<RHS> for Queriable<F> {
        type Output = Expr<F>;

        fn sub(self, rhs: RHS) -> Self::Output {
            self.expr() - rhs
        }
    }

    impl<F: Clone, RHS: Into<Expr<F>>> Mul<RHS> for Queriable<F> {
        type Output = Expr<F>;

        fn mul(self, rhs: RHS) -> Self::Output {
            self.expr() * rhs
        }
    }

    impl<F: Clone> Neg for Queriable<F> {
        type Output = Expr<F>;

        fn neg(self) -> Self::Output {
            self.expr().neg()
        }
    }

    impl<F, RHS: Into<Expr<F>>> BitOr<RHS> for Queriable<F> {
        type Output = Expr<F>;
        fn bitor(self, rhs: RHS) -> Self::Output {
            Expr::Equal(Box::new(self.into()), Box::new(rhs.into()))
        }
    }
}
