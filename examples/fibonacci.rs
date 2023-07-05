use std::hash::Hash;

use chiquito::{
    ast::expr::*,
    backend::halo2::{chiquito2Halo2, ChiquitoHalo2}, /* compiles to Chiquito Halo2 backend,
                                                      * which can be integrated into Halo2
                                                      * circuit */
    compiler::{
        cell_manager::SingleRowCellManager, // input for constructing the compiler
        step_selector::SimpleStepSelectorBuilder, // input for constructing the compiler
        Compiler,
    },
    dsl::{
        cb::*,   // functions for constraint building
        circuit, // main function for constructing an AST circuit
    },
    ir::Circuit,
    wit_gen::TraceGenerator,
};
use halo2_proofs::{
    arithmetic::Field, circuit::SimpleFloorPlanner, dev::MockProver, halo2curves::bn256::Fr,
    plonk::ConstraintSystem,
};

// the main circuit function: returns the compiled IR of a Chiquito circuit
// Generic types F, (), (u64, 64) stand for:
// 1. type that implements a field trait
// 2. empty trace arguments, i.e. (), because there are no external inputs to the Chiquito circuit
// 3. two witness generation arguments both of u64 type, i.e. (u64, u64)
fn fibo_circuit<F: Field + From<u64> + Hash>() -> (Circuit<F>, Option<TraceGenerator<F, ()>>) {
    // PLONKish table for the Fibonacci circuit:
    // | a | b | c |
    // | 1 | 1 | 2 |
    // | 1 | 2 | 3 |
    // | 2 | 3 | 5 |
    // | 3 | 5 | 8 |
    // ...
    let fibo = circuit::<F, (), (u64, u64), _>("fibonacci", |ctx| {
        // the following objects (forward signals, steptypes) are defined on the circuit-level

        // forward signals can have constraints across different steps
        let a = ctx.forward("a");
        let b = ctx.forward("b");

        // initiate step type for future definition
        let fibo_step = ctx.step_type("fibo step");
        let fibo_last_step = ctx.step_type("last step");

        // enforce step type of the first step
        ctx.pragma_first_step(fibo_step);
        // enforce step type of the last step
        ctx.pragma_last_step(fibo_last_step);

        // define step type
        let fibo_step = ctx.step_type_def(fibo_step, |ctx| {
            // the following objects (constraints, transition constraints, witness generation
            // function) are defined on the step type-level

            // internal signals can only have constraints within the same step
            let c = ctx.internal("c");

            // in setup we define the constraints of the step
            ctx.setup(move |ctx| {
                // regular constraints are for internal signals only
                // constrain that a + b == c by calling `eq` function from constraint builder
                ctx.constr(eq(a + b, c));

                // transition constraints accepts forward signals as well
                // constrain that b is equal to the next instance of a, by calling `next` on forward
                // signal
                ctx.transition(eq(b, a.next()));
                // constrain that c is equal to the next instance of c, by calling `next` on forward
                // signal
                ctx.transition(eq(c, b.next()));
            });

            // witness generation (wg) function is Turing complete and allows arbitrary user defined
            // logics for assigning witness values wg function is defined here but no
            // witness value is assigned yet
            ctx.wg(move |ctx, (a_value, b_value): (u32, u32)| {
                println!("fib line wg: {} {} {}", a_value, b_value, a_value + b_value);
                // assign arbitrary input values from witness generation function to witnesses
                ctx.assign(a, a_value.field());
                ctx.assign(b, b_value.field());
                ctx.assign(c, (a_value + b_value).field());
            })
        });

        let fibo_last_step = ctx.step_type_def(fibo_last_step, |ctx| {
            let c = ctx.internal("c");

            ctx.setup(move |ctx| {
                // constrain that a + b == c by calling `eq` function from constraint builder
                // no transition constraint needed for the next instances of a and b, because it's
                // the last step
                ctx.constr(eq(a + b, c));
            });

            ctx.wg(move |ctx, (a_value, b_value): (u32, u32)| {
                println!(
                    "fib last line wg: {} {} {}",
                    a_value,
                    b_value,
                    a_value + b_value
                );
                ctx.assign(a, a_value.field());
                ctx.assign(b, b_value.field());
                ctx.assign(c, (a_value + b_value).field());
            })
        });

        // trace function is responsible for adding step instantiations defined in step_type_def
        // function above trace function is Turing complete and allows arbitrary user
        // defined logics for assigning witness values
        ctx.trace(move |ctx: _, _| {
            // add function adds a step instantiation to the main circuit and calls witness
            // generation function defined in step_type_def input values for witness
            // generation function are (1, 1) in this step instance
            ctx.add(&fibo_step, (1, 1));
            let mut a = 1;
            let mut b = 2;

            for _i in 1..10 {
                ctx.add(&fibo_step, (a, b));

                let prev_a = a;
                a = b;
                b += prev_a;
            }

            ctx.add(&fibo_last_step, (a, b));
        })
    });

    let compiler = Compiler::new(SingleRowCellManager {}, SimpleStepSelectorBuilder {});

    compiler.compile(&fibo)
}

// After compiling Chiquito AST to an IR, it is further parsed by a Chiquito Halo2 backend and
// integrated into a Halo2 circuit, which is done by the boilerplate code below.

// *** Halo2 boilerplate ***
#[derive(Clone)]
struct FiboConfig<F: Field + From<u64>> {
    // ChiquitoHalo2 object in the bytecode circuit config struct
    compiled: ChiquitoHalo2<F>,
    wit_gen: TraceGenerator<F, ()>,
}

impl<F: Field + From<u64> + Hash> FiboConfig<F> {
    fn new(meta: &mut ConstraintSystem<F>) -> FiboConfig<F> {
        let (chiquito, wit_gen) = fibo_circuit::<F>();

        // chiquito2Halo2 function in Halo2 backend can convert ir::Circuit object to a
        // ChiquitoHalo2 object, which can be further integrated into a Halo2 circuit in the
        // example below
        let mut compiled = chiquito2Halo2(chiquito);

        // ChiquitoHalo2 objects have their own configure and synthesize functions defined in the
        // Chiquito Halo2 backend
        compiled.configure(meta);

        FiboConfig {
            compiled,
            wit_gen: wit_gen.expect("trace generator not returned"),
        }
    }
}

#[derive(Default)]
struct FiboCircuit {}

// integrate Chiquito circuit into a Halo2 circuit
impl<F: Field + From<u64> + Hash> halo2_proofs::plonk::Circuit<F> for FiboCircuit {
    type Config = FiboConfig<F>;
    type Params = ();

    type FloorPlanner = SimpleFloorPlanner;

    // function in Halo2 circuit interface
    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    // function in Halo2 circuit interface
    fn configure(meta: &mut halo2_proofs::plonk::ConstraintSystem<F>) -> Self::Config {
        FiboConfig::<F>::new(meta)
    }

    // function in Halo2 circuit interface
    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl halo2_proofs::circuit::Layouter<F>,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        // ChiquitoHalo2 objects have their own configure and synthesize functions defined in the
        // Chiquito Halo2 backend
        config
            .compiled
            .synthesize(&mut layouter, Some(config.wit_gen.generate(())));

        Ok(())
    }
}

// standard main function for a Halo2 circuit
fn main() {
    let circuit = FiboCircuit {};

    let prover = MockProver::<Fr>::run(7, &circuit, Vec::new()).unwrap();

    let result = prover.verify_par();

    println!("{:#?}", result);

    if let Err(failures) = &result {
        for failure in failures.iter() {
            println!("{}", failure);
        }
    }

    // plaf boilerplate
    use chiquito::backend::plaf::chiquito2Plaf;
    use polyexen::plaf::{backends::halo2::PlafH2Circuit, WitnessDisplayCSV};

    // get Chiquito ir
    let (circuit, wit_gen) = fibo_circuit::<Fr>();
    // get Plaf
    let (plaf, plaf_wit_gen) = chiquito2Plaf(circuit, 8, false);
    let wit = plaf_wit_gen.generate(wit_gen.map(|v| v.generate(())));

    // debug only: print witness
    println!("{}", WitnessDisplayCSV(&wit));

    // get Plaf halo2 circuit from Plaf's halo2 backend
    // this is just a proof of concept, because Plaf only has backend for halo2
    // this is unnecessary because Chiquito has a halo2 backend already
    let plaf_circuit = PlafH2Circuit { plaf, wit };

    // same as halo2 boilerplate above
    let prover_plaf = MockProver::<Fr>::run(8, &plaf_circuit, Vec::new()).unwrap();

    let result_plaf = prover_plaf.verify_par();

    println!("result = {:#?}", result);

    if let Err(failures) = &result_plaf {
        for failure in failures.iter() {
            println!("{}", failure);
        }
    }
}
