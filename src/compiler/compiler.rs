use std::{collections::HashMap, hash::Hash, marker::PhantomData};

use num_bigint::BigInt;

use crate::{
    field::Field,
    frontend::dsl::{
        cb::{Constraint, Typing},
        circuit, CircuitContext, StepTypeContext, StepTypeHandler,
    },
    parser::{
        ast::{tl::TLDecl, Identifiable, Identifier},
        lang::TLDeclsParser,
    },
    poly::{self, mielim::mi_elimination, reduce::reduce_degree, Expr},
    sbpir::{query::Queriable, ForwardSignal, InternalSignal, SBPIR},
    util::UUID,
};

use super::{
    semantic::{SymTable, SymbolCategory},
    setup_inter::{interpret, Setup},
    Config, Message, Messages,
};

/// This compiler compiles from chiquito source code to the SBPIR.
#[derive(Default)]
pub(super) struct Compiler<F> {
    pub(super) config: Config,

    messages: Vec<Message>,

    symbol_uuid: HashMap<(String, String), UUID>,

    forward_signals: HashMap<UUID, ForwardSignal>,
    internal_signals: HashMap<UUID, InternalSignal>,
    step_type_handler: HashMap<UUID, StepTypeHandler>,

    _p: PhantomData<F>,
}

impl<F: Field + Hash> Compiler<F> {
    /// Creates a configured compiler.
    pub fn new(config: Config) -> Self {
        Compiler {
            config,
            ..Compiler::default()
        }
    }

    /// Compile the source code.
    pub(super) fn compile(&mut self, source: &str) -> Result<SBPIR<F, ()>, ()> {
        let ast = self.parse(source)?;
        let symbols = self.semantic(&ast)?;
        let setup = Self::interpret(&ast, &symbols);
        let setup = Self::map_consts(setup);
        let circuit = self.build(&setup, &symbols);
        let circuit = Self::mi_elim(circuit);
        let circuit = if let Some(degree) = self.config.max_degree {
            Self::reduce(circuit, degree)
        } else {
            circuit
        };

        Ok(circuit)
    }

    /// Get all messages from the compilation.
    pub(super) fn get_messages(self) -> Vec<Message> {
        self.messages
    }

    fn parse(&mut self, source: &str) -> Result<Vec<TLDecl<BigInt, Identifier>>, ()> {
        let result = TLDeclsParser::new().parse(source);

        match result {
            Ok(ast) => Ok(ast),
            Err(error) => {
                self.messages.push(Message::ParseErr {
                    msg: error.to_string(),
                });
                Err(())
            }
        }
    }

    fn semantic(&mut self, ast: &[TLDecl<BigInt, Identifier>]) -> Result<SymTable, ()> {
        let result = super::semantic::analyser::analyse(ast);
        let has_errors = result.messages.has_errors();

        self.messages.extend(result.messages);

        if has_errors {
            Err(())
        } else {
            Ok(result.symbols)
        }
    }

    fn interpret(
        ast: &[TLDecl<BigInt, Identifier>],
        symbols: &SymTable,
    ) -> Setup<BigInt, Identifier> {
        interpret(ast, symbols)
    }

    fn map_consts(setup: Setup<BigInt, Identifier>) -> Setup<F, Identifier> {
        setup
            .iter()
            .map(|(machine_id, machine)| {
                let new_machine: HashMap<String, Vec<Expr<F, Identifier, ()>>> = machine
                    .iter()
                    .map(|(step_id, step)| {
                        let new_step = step.iter().map(|pi| Self::map_pi_consts(pi)).collect();

                        (step_id.clone(), new_step)
                    })
                    .collect();

                (machine_id.clone(), new_machine)
            })
            .collect()
    }

    fn map_pi_consts(expr: &Expr<BigInt, Identifier, ()>) -> Expr<F, Identifier, ()> {
        use Expr::*;
        match expr {
            Const(v, _) => Const(F::from_big_int(v), ()),
            Sum(ses, _) => Sum(ses.iter().map(|se| Self::map_pi_consts(se)).collect(), ()),
            Mul(ses, _) => Mul(ses.iter().map(|se| Self::map_pi_consts(se)).collect(), ()),
            Neg(se, _) => Neg(Box::new(Self::map_pi_consts(se)), ()),
            Pow(se, exp, _) => Pow(Box::new(Self::map_pi_consts(se)), *exp, ()),
            Query(q, _) => Query(q.clone(), ()),
            Halo2Expr(_, _) => todo!(),
            MI(se, _) => MI(Box::new(Self::map_pi_consts(se)), ()),
        }
    }

    fn build(&mut self, setup: &Setup<F, Identifier>, symbols: &SymTable) -> SBPIR<F, ()> {
        circuit("circuit", |ctx| {
            for (machine_id, machine) in setup {
                self.add_forwards(ctx, symbols, machine_id);
                self.add_step_type_handlers(ctx, symbols, machine_id);

                for state_id in machine.keys() {
                    ctx.step_type_def(self.get_step_type_handler(machine_id, state_id), |ctx| {
                        self.add_internals(ctx, symbols, machine_id, state_id);

                        ctx.setup(|ctx| {
                            let pis = self.translate_queries(symbols, setup, machine_id, state_id);
                            pis.iter().for_each(|pi| {
                                let constraint = Constraint {
                                    annotation: format!("{:?}", pi),
                                    expr: pi.clone(),
                                    typing: Typing::AntiBooly,
                                };
                                ctx.constr(constraint);
                            });
                        });

                        ctx.wg(|_, _: ()| {})
                    });
                }
            }
        })
    }

    fn mi_elim(mut circuit: SBPIR<F, ()>) -> SBPIR<F, ()> {
        for (_, step_type) in circuit.step_types.iter_mut() {
            let mut signal_factory = SignalFactory::default();

            step_type.decomp_constraints(|expr| mi_elimination(expr.clone(), &mut signal_factory));
        }

        circuit
    }

    fn reduce(mut circuit: SBPIR<F, ()>, degree: usize) -> SBPIR<F, ()> {
        for (_, step_type) in circuit.step_types.iter_mut() {
            let mut signal_factory = SignalFactory::default();

            step_type.decomp_constraints(|expr| {
                reduce_degree(expr.clone(), degree, &mut signal_factory)
            });
        }

        circuit
    }

    fn translate_queries(
        &mut self,
        symbols: &SymTable,
        setup: &Setup<F, Identifier>,
        machine_id: &str,
        state_id: &str,
    ) -> Vec<Expr<F, Queriable<F>, ()>> {
        let exprs = setup.get(machine_id).unwrap().get(state_id).unwrap();

        exprs
            .iter()
            .map(|expr| self.translate_queries_expr(symbols, machine_id, state_id, expr))
            .collect()
    }

    fn translate_queries_expr(
        &mut self,
        symbols: &SymTable,
        machine_id: &str,
        state_id: &str,
        expr: &Expr<F, Identifier, ()>,
    ) -> Expr<F, Queriable<F>, ()> {
        use Expr::*;
        match expr {
            Const(v, _) => Const(*v, ()),
            Sum(ses, _) => Sum(
                ses.iter()
                    .map(|se| self.translate_queries_expr(symbols, machine_id, state_id, se))
                    .collect(),
                (),
            ),
            Mul(ses, _) => Mul(
                ses.iter()
                    .map(|se| self.translate_queries_expr(symbols, machine_id, state_id, se))
                    .collect(),
                (),
            ),
            Neg(se, _) => Neg(
                Box::new(self.translate_queries_expr(symbols, machine_id, state_id, se.as_ref())),
                (),
            ),
            Pow(se, exp, _) => Pow(
                Box::new(self.translate_queries_expr(symbols, machine_id, state_id, se.as_ref())),
                *exp,
                (),
            ),
            MI(se, _) => MI(
                Box::new(self.translate_queries_expr(symbols, machine_id, state_id, se.as_ref())),
                (),
            ),
            Halo2Expr(se, _) => Halo2Expr(se.clone(), ()),
            Query(id, _) => Query(self.translate_query(symbols, machine_id, state_id, id), ()),
        }
    }

    fn translate_query(
        &mut self,
        symbols: &SymTable,
        machine_id: &str,
        state_id: &str,
        id: &Identifier,
    ) -> Queriable<F> {
        use super::semantic::{ScopeCategory, SymbolCategory::*};

        let symbol = symbols
            .find_symbol(
                &[
                    "/".to_string(),
                    machine_id.to_string(),
                    state_id.to_string(),
                ],
                id.name(),
            )
            .unwrap_or_else(|| panic!("semantic analyser fail: undeclared id {}", id.name()));

        match symbol.symbol.category {
            InputSignal | OutputSignal | InoutSignal => {
                self.translate_forward_queriable(machine_id, id)
            }
            Signal => match symbol.scope {
                ScopeCategory::Machine => self.translate_forward_queriable(machine_id, id),
                ScopeCategory::State => {
                    if id.rotation() != 0 {
                        unreachable!("semantic analyser should prevent this");
                    }
                    let signal =
                        self.get_internal(&format!("//{}/{}", machine_id, state_id), &id.name());

                    Queriable::Internal(signal)
                }

                ScopeCategory::Global => unreachable!("no global signals"),
            },

            State => Queriable::StepTypeNext(self.get_step_type_handler(machine_id, &id.name())),

            _ => unreachable!("semantic analysis should prevent this"),
        }
    }

    fn translate_forward_queriable(&mut self, machine_id: &str, id: &Identifier) -> Queriable<F> {
        let forward = self.get_forward(machine_id, &id.name());
        let rot = if id.rotation() == 0 {
            false
        } else if id.rotation() == 1 {
            true
        } else {
            unreachable!("semantic analyser should prevent this")
        };

        Queriable::Forward(forward, rot)
    }

    fn get_forward(&mut self, machine_id: &str, forward_id: &str) -> ForwardSignal {
        let scope_name = format!("//{}", machine_id);

        let uuid = self
            .symbol_uuid
            .get(&(scope_name, forward_id.to_string()))
            .expect("semantic analyser fail: forward should exist");

        *self.forward_signals.get(uuid).unwrap()
    }

    fn get_internal(&mut self, scope_name: &str, symbol_name: &str) -> InternalSignal {
        let uuid = self
            .symbol_uuid
            .get(&(scope_name.to_string(), symbol_name.to_string()))
            .expect("semantic analyser fail");

        *self.internal_signals.get(uuid).unwrap()
    }

    fn get_step_type_handler(&mut self, machine_id: &str, state_id: &str) -> StepTypeHandler {
        let scope_name = format!("//{}", machine_id);
        let uuid = self
            .symbol_uuid
            .get(&(scope_name, state_id.to_string()))
            .expect("semantic analyser fail");

        *self.step_type_handler.get(uuid).unwrap()
    }

    fn get_all_internals(
        &mut self,
        symbols: &SymTable,
        machine_id: &str,
        state_id: &str,
    ) -> Vec<String> {
        let symbols = symbols
            .get_scope(&[
                "/".to_string(),
                machine_id.to_string(),
                state_id.to_string(),
            ])
            .expect("scope not found")
            .get_symbols();

        symbols
            .iter()
            .filter(|(_, entry)| entry.category == SymbolCategory::Signal)
            .map(|(id, _)| id)
            .cloned()
            .collect()
    }

    fn add_internals(
        &mut self,
        ctx: &mut StepTypeContext<F>,
        symbols: &SymTable,
        machine_id: &str,
        state_id: &str,
    ) {
        let internal_ids = self.get_all_internals(symbols, machine_id, state_id);
        let scope_name = format!("//{}/{}", machine_id, state_id);

        for internal_id in internal_ids {
            let name = format!("{}:{}", &scope_name, internal_id);

            let queriable = ctx.internal(name.as_str());
            if let Queriable::Internal(signal) = queriable {
                self.symbol_uuid
                    .insert((scope_name.clone(), internal_id), signal.uuid());
                self.internal_signals.insert(signal.uuid(), signal);
            } else {
                unreachable!("ctx.internal returns not internal signal");
            }
        }
    }

    fn add_step_type_handlers(
        &mut self,
        ctx: &mut CircuitContext<F, ()>,
        symbols: &SymTable,
        machine_id: &str,
    ) {
        let symbols = symbols
            .get_scope(&["/".to_string(), machine_id.to_string()])
            .expect("scope not found")
            .get_symbols();

        let state_ids: Vec<_> = symbols
            .iter()
            .filter(|(_, entry)| entry.category == SymbolCategory::State)
            .map(|(id, _)| id)
            .cloned()
            .collect();

        for state_id in state_ids {
            let scope_name = format!("//{}", machine_id);
            let name = format!("{}:{}", scope_name, state_id);

            let handler = ctx.step_type(&name);
            self.step_type_handler.insert(handler.uuid(), handler);
            self.symbol_uuid
                .insert((scope_name, state_id), handler.uuid());
        }
    }

    fn add_forwards(
        &mut self,
        ctx: &mut CircuitContext<F, ()>,
        symbols: &SymTable,
        machine_id: &str,
    ) {
        println!("{:?}", symbols);
        let symbols = symbols
            .get_scope(&["/".to_string(), machine_id.to_string()])
            .expect("scope not found")
            .get_symbols();

        let forward_ids: Vec<_> = symbols
            .iter()
            .filter(|(_, entry)| entry.is_signal())
            .map(|(id, _)| id)
            .cloned()
            .collect();

        for forward_id in forward_ids {
            let scope_name = format!("//{}", machine_id);
            let name = format!("{}:{}", scope_name, forward_id);

            let queriable = ctx.forward(name.as_str());
            if let Queriable::Forward(signal, _) = queriable {
                self.symbol_uuid
                    .insert((scope_name, forward_id), signal.uuid());
                self.forward_signals.insert(signal.uuid(), signal);
            } else {
                unreachable!("ctx.internal returns not internal signal");
            }
        }
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
    use halo2_proofs::halo2curves::bn256::Fr;

    use crate::compiler::compile;

    use super::Config;

    #[test]
    fn test_compiler_fibo() {
        let circuit = "
        machine fibo(signal n) (signal b: field) {
            // n and be are created automatically as shared
            // signals
            signal a: field, i;

            // there is always a state called initial
            // input signals get binded to the signal
            // in the initial state (first instance)
            state initial {
             signal c;

             i, a, b, c <== 1, 1, 1, 2;

             -> middle {
              a', b', n' <== b, c, n;
             }
            }

            state middle {
             signal c;

             c <== a + b;

             if i + 1 == n {
              -> final {
               i', b', n' <== i + 1, c, n;
              }
             } else {
              -> middle {
               i', a', b', n' <== i + 1, b, c, n;
              }
             }
            }

            // There is always a state called final.
            // Output signals get automatically bindinded to the signals
            // with the same name in the final step (last instance).
            // This state can be implicit if there are no constraints in it.
           }
        ";

        let result = compile::<Fr>(circuit, Config::default().max_degree(2));

        match result.0 {
            Ok(sbpir) => println!("{:#?}", sbpir),
            Err(_) => println!("{:#?}", result.1),
        }
    }
}