use chiquito::{
    ast::{query::Queriable, Expr},
    frontend::dsl::{lb::LookupTable, super_circuit, CircuitContext, StepTypeWGHandler},
    plonkish::{
        backend::halo2::{chiquitoSuperCircuit2Halo2, ChiquitoHalo2SuperCircuit},
        compiler::{
            cell_manager::{MaxWidthCellManager, SingleRowCellManager},
            config,
            step_selector::SimpleStepSelectorBuilder,
        },
        ir::sc::SuperCircuit,
    },
};
use std::hash::Hash;

use halo2_proofs::{
    dev::MockProver,
    halo2curves::{bn256::Fr, group::ff::PrimeField}, circuit,
};

use std::{
    fs::File,
    io::{self, Write},
};

const BIT_COUNT: usize = 3;
const PART_SIZE: usize = 5;
const NUM_BYTES_PER_WORD: usize = 8;
const NUM_BITS_PER_BYTE: usize = 8;
const NUM_WORDS_TO_ABSORB: usize = 17;
const RATE: usize = NUM_WORDS_TO_ABSORB * NUM_BYTES_PER_WORD;
const NUM_BITS_PER_WORD: usize = NUM_BYTES_PER_WORD * NUM_BITS_PER_BYTE;
const NUM_PER_WORD: usize = NUM_BYTES_PER_WORD * NUM_BITS_PER_BYTE / 2;
const RATE_IN_BITS: usize = RATE * NUM_BITS_PER_BYTE;
const NUM_ROUNDS: usize = 24;
const BIT_SIZE: usize = 2usize.pow(BIT_COUNT as u32);

pub const ROUND_CST: [u64; NUM_ROUNDS + 1] = [
    0x0000000000000001,
    0x0000000000008082,
    0x800000000000808a,
    0x8000000080008000,
    0x000000000000808b,
    0x0000000080000001,
    0x8000000080008081,
    0x8000000000008009,
    0x000000000000008a,
    0x0000000000000088,
    0x0000000080008009,
    0x000000008000000a,
    0x000000008000808b,
    0x800000000000008b,
    0x8000000000008089,
    0x8000000000008003,
    0x8000000000008002,
    0x8000000000000080,
    0x000000000000800a,
    0x800000008000000a,
    0x8000000080008081,
    0x8000000000008080,
    0x0000000080000001,
    0x8000000080008008,
    0x0000000000000000,
];

pub const XOR_VALUE: [u64; NUM_BITS_PER_WORD] = [
    0x0, 0x1, 0x0, 0x1, 0x0, 0x1, 0x0, 0x1, 0x8, 0x9, 0x8, 0x9, 0x8, 0x9, 0x8, 0x9, 0x0, 0x1, 0x0,
    0x1, 0x0, 0x1, 0x0, 0x1, 0x8, 0x9, 0x8, 0x9, 0x8, 0x9, 0x8, 0x9, 0x0, 0x1, 0x0, 0x1, 0x0, 0x1,
    0x0, 0x1, 0x8, 0x9, 0x8, 0x9, 0x8, 0x9, 0x8, 0x9, 0x0, 0x1, 0x0, 0x1, 0x0, 0x1, 0x0, 0x1, 0x8,
    0x9, 0x8, 0x9, 0x8, 0x9, 0x8, 0x9,
];

pub const CHI_VALUE: [u64; NUM_BITS_PER_WORD] = [
    0x0, 0x1, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x8, 0x9, 0x9, 0x8, 0x8, 0x8, 0x8, 0x8, 0x8, 0x9, 0x9,
    0x8, 0x8, 0x8, 0x8, 0x8, 0x0, 0x1, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x1, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x1, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x1, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0,
];

/// Pack bits in the range [0,BIT_SIZE[ into a sparse keccak word
pub(crate) fn pack<F: PrimeField>(bits: &[u8]) -> F {
    pack_with_base(bits, BIT_SIZE)
}

/// Pack bits in the range [0,BIT_SIZE[ into a sparse keccak word with the
/// specified bit base
pub(crate) fn pack_with_base<F: PrimeField>(bits: &[u8], base: usize) -> F {
    // \sum 8^i * bit_i
    let base = F::from(base as u64);
    bits.iter()
        .rev()
        .fold(F::ZERO, |acc, &bit| acc * base + F::from(bit as u64))
}

/// Calculates a ^ b with a and b field elements
pub(crate) fn field_xor<F: PrimeField<Repr = [u8; 32]>>(a: F, b: F) -> F {
    let mut bytes = [0u8; 32];
    for (idx, (a, b)) in a
        .to_repr()
        .as_ref()
        .iter()
        .zip(b.to_repr().as_ref().iter())
        .enumerate()
    {
        bytes[idx] = *a ^ *b;
    }
    F::from_repr(bytes).unwrap()
}

pub(crate) fn pack_u64<F: PrimeField>(value: u64) -> F {
    pack(
        &((0..NUM_BITS_PER_WORD)
            .map(|i| ((value >> i) & 1) as u8)
            .collect::<Vec<_>>()),
    )
}

fn convert_field_to_vec_bits<F: PrimeField>(value: F) -> Vec<u8> {
    let mut v_vec = Vec::new();
    let mut left = 0;
    for (idx, &v1) in value.to_repr().as_ref().iter().enumerate() {
        if idx % 3 == 0 {
            v_vec.push(v1 % 8);
            v_vec.push((v1 / 8) % 8);
            left = v1 / 64;
        } else if idx % 3 == 1 {
            v_vec.push((v1 % 2) * 4 + left);
            v_vec.push((v1 / 2) % 8);
            v_vec.push((v1 / 16) % 8);
            left = v1 / 128;
        } else {
            v_vec.push((v1 % 4) * 2 + left);
            v_vec.push((v1 / 4) % 8);
            v_vec.push(v1 / 32);
            left = 0;
        }
    }
    v_vec[0..64].to_vec()
}

fn convert_bits_to_f<F: PrimeField<Repr = [u8; 32]>>(value_vec: &Vec<u8>) -> F {
    assert_eq!(value_vec.len(), NUM_BITS_PER_WORD);
    let mut sum_value_arr: Vec<u8> = (0..24)
        .map(|t| {
            if t % 3 == 0 {
                value_vec[(t / 3) * 8]
                    + value_vec[(t / 3) * 8 + 1] * 8
                    + (value_vec[(t / 3) * 8 + 2] % 4) * 64
            } else if t % 3 == 1 {
                value_vec[(t / 3) * 8 + 2] / 4
                    + value_vec[(t / 3) * 8 + 3] * 2
                    + (value_vec[(t / 3) * 8 + 4]) * 16
                    + ((value_vec[(t / 3) * 8 + 5]) % 2) * 128
            } else {
                value_vec[(t / 3) * 8 + 5] / 2
                    + value_vec[(t / 3) * 8 + 6] * 4
                    + (value_vec[(t / 3) * 8 + 7]) * 32
            }
        })
        .collect();
    while sum_value_arr.len() < 32 {
        sum_value_arr.push(0);
    }
    F::from_repr(sum_value_arr.try_into().unwrap()).unwrap()
}

struct KeccakFThetaValue<F: PrimeField<Repr = [u8; 32]>> {
    value_rows: Vec<(F, F)>,
    sum_row_vec: Vec<(F, F)>,
    xor_vec: Vec<(Vec<F>, Vec<F>)>,
    sum_xor_vec: Vec<(Vec<F>, Vec<F>, F, F)>,
}

fn eval_keccak_f_theta_step<F: PrimeField<Repr = [u8; 32]>>(
    s_new: &mut [[F; PART_SIZE]; PART_SIZE],
) -> KeccakFThetaValue<F> {
    // Theta
    // a[x][y][z] = a[x][y][z] + \sum_y' a[x-1][y'][z] + \sum a[x+1][y'][z-1]
    let s: Vec<Vec<F>> = s_new.iter().map(|s_new| s_new.to_vec()).collect();
    let mut value_rows = Vec::new();
    let mut sum_row_vec = Vec::new();
    let mut xor_vec = Vec::new();
    let mut sum_xor_vec = Vec::new();
    for s in s.iter().take(PART_SIZE) {
        // value = \sum_y' a[x][y'][z]
        let sum = s[0] + s[1] + s[2] + s[3] + s[4];
        let sum_bits = convert_field_to_vec_bits(sum);

        let xor: F = field_xor(
            field_xor(field_xor(field_xor(s[0], s[1]), s[2]), s[3]),
            s[4],
        );
        // rot_value = \sum_y' a[x][y'][z-1]
        let xor_bits = convert_field_to_vec_bits(xor);
        let mut xor_bits_move = xor_bits.clone();
        xor_bits_move.rotate_right(1);
        let xor_rot = convert_bits_to_f(&xor_bits_move);

        sum_row_vec.push((xor, xor_rot));

        let mut sum_split = Vec::new();
        let mut sum_split_xor = Vec::new();
        for i in 0..sum_bits.len() / 2 {
            sum_split.push(
                F::from_u128(sum_bits[2 * i] as u128)
                    + F::from_u128(sum_bits[2 * i + 1] as u128) * F::from_u128(8),
            );
            sum_split_xor.push(
                F::from_u128(xor_bits[2 * i] as u128)
                    + F::from_u128(xor_bits[2 * i + 1] as u128) * F::from_u128(8),
            );
        }
        sum_xor_vec.push((
            sum_split,
            sum_split_xor,
            F::from_u128(xor_bits[xor_bits.len() - 2] as u128),
            F::from_u128(xor_bits[xor_bits.len() - 1] as u128),
        ));
    }

    for i in 0..PART_SIZE {
        let value = sum_row_vec[(i + PART_SIZE - 1) % PART_SIZE].0;
        let rot_value = sum_row_vec[(i + 1) % PART_SIZE].1;
        for j in 0..PART_SIZE {
            let st = s[i][j] + value + rot_value;
            s_new[i][j] = field_xor(field_xor(s[i][j], value), rot_value);
            value_rows.push((s[i][j], s_new[i][j]));

            let mut st_split = Vec::new();
            let mut st_split_xor = Vec::new();
            let st_bit_vec = convert_field_to_vec_bits(st);
            let st_bit_xor_vec = convert_field_to_vec_bits(s_new[i][j]);
            for i in 0..st_bit_vec.len() / 2 {
                st_split.push(
                    F::from_u128(st_bit_vec[2 * i] as u128)
                        + F::from_u128(st_bit_vec[2 * i + 1] as u128) * F::from_u128(8),
                );
                st_split_xor.push(
                    F::from_u128(st_bit_xor_vec[2 * i] as u128)
                        + F::from_u128(st_bit_xor_vec[2 * i + 1] as u128) * F::from_u128(8),
                );
            }
            xor_vec.push((st_split, st_split_xor));
        }
    }

    KeccakFThetaValue {
        value_rows,
        sum_row_vec,
        xor_vec,
        sum_xor_vec,
    }
}

struct KeccakFRHOValue<F: PrimeField<Repr = [u8; 32]>> {
    absorb_rows: Vec<(F, F)>,
    xor_rows: Vec<(Vec<F>, F, F)>,
}

fn eval_keccak_f_rho_step<F: PrimeField<Repr = [u8; 32]>>(
    s_new: &mut [[F; PART_SIZE]; PART_SIZE],
) -> KeccakFRHOValue<F> {
    // rho
    // a[x][y][z] = a[x][y][z-(t+1)(t+2)/2]
    let s: Vec<Vec<F>> = s_new.iter().map(|s_new| s_new.to_vec()).collect();
    let mut absorb_rows = Vec::new();
    let mut xor_rows = Vec::new();
    let mut i = 1;
    let mut j = 0;
    for t in 0..25 {
        if t == 0 {
            i = 0;
            j = 0
        } else if t == 1 {
            i = 1;
            j = 0;
        } else {
            let m = j;
            j = (2 * i + 3 * j) % 5;
            i = m;
        }
        let v = (t * (t + 1) / 2) % 64;

        let mut v_vec = convert_field_to_vec_bits(s[i][j]);

        // xor_rows.push(v_vec.iter().map(|&v| F::from_u128(v as u128)).collect());
        let mut b0 = F::ZERO;
        let mut b1 = F::ZERO;
        if v % 2 != 0 {
            b0 = F::from(v_vec[v_vec.len() - v - 1] as u64);
            b1 = F::from(v_vec[v_vec.len() - v] as u64);
        }
        xor_rows.push((
            (0..v_vec.len() / 2)
                .map(|i| {
                    F::from_u128(v_vec[2 * i] as u128)
                        + F::from_u128(v_vec[2 * i + 1] as u128) * F::from_u128(8)
                })
                .collect(),
            b0,
            b1,
        ));
        v_vec.rotate_right(v);

        s_new[i][j] = convert_bits_to_f(&v_vec);
    }
    for i in 0..PART_SIZE {
        for j in 0..PART_SIZE {
            absorb_rows.push((s[i][j], s_new[i][j]));
        }
    }
    KeccakFRHOValue {
        absorb_rows,
        xor_rows,
    }
}

fn eval_keccak_f_pi_step<F: PrimeField<Repr = [u8; 32]>>(s_new: &mut [[F; PART_SIZE]; PART_SIZE]) {
    // pi
    // a[y][2x + 3y] = a[x][y]
    let s: Vec<Vec<F>> = s_new.iter().map(|s_new| s_new.to_vec()).collect();
    let mut i = 1;
    let mut j = 0;
    for _ in 0..PART_SIZE * PART_SIZE {
        let x = j;
        let y = (2 * i + 3 * j) % 5;
        s_new[x][y] = s[i][j];
        i = x;
        j = y;
    }
}

fn eval_keccak_f_chi_step<F: PrimeField<Repr = [u8; 32]>>(
    s_new: &mut [[F; PART_SIZE]; PART_SIZE],
) -> Vec<(F, F, F, F)> {
    // chi
    // a[x] = a[x] ^ (~a[x+1] & a[x+2])
    let s: Vec<Vec<F>> = s_new.iter().map(|s_new| s_new.to_vec()).collect();
    let mut absorb_rows = Vec::new();

    for i in 0..PART_SIZE {
        for j in 0..PART_SIZE {
            let a_vec = convert_field_to_vec_bits(s[i][j]);
            let b_vec = convert_field_to_vec_bits(s[(i + 1) % 5][j]);
            let c_vec = convert_field_to_vec_bits(s[(i + 2) % 5][j]);
            let sum_vec: Vec<u8> = a_vec
                .iter()
                .zip(b_vec.iter().zip(c_vec.iter()))
                .map(|(&a, (&b, &c))| 3 - 2 * a + b - c)
                .collect();

            let sum = convert_bits_to_f(&sum_vec);

            let split_chi_value: Vec<u8> = sum_vec
                .iter()
                .map(|&v| if v == 1 || v == 2 { 1 } else { 0 })
                .collect();
            s_new[i][j] = convert_bits_to_f(&split_chi_value);

            absorb_rows.push((s[i][j], sum, s_new[i][j], F::ZERO));
        }
    }
    absorb_rows
}

fn eval_keccak_f_iota_step<F: PrimeField<Repr = [u8; 32]>>(
    s: F,
    round_cst: u64,
) -> (Vec<(F, F)>, F) {
    // iota

    let s_vec = convert_field_to_vec_bits(s);
    let cst_vec = convert_field_to_vec_bits(pack_u64::<F>(round_cst));

    let split_xor_vec: Vec<u8> = s_vec
        .iter()
        .zip(cst_vec.iter())
        .map(|(v1, v2)| v1 ^ v2)
        .collect();
    let xor_rows: Vec<(F, F)> = s_vec
        .iter()
        .zip(cst_vec.iter())
        .map(|(v1, v2)| {
            (
                F::from_u128((v1 + v2) as u128),
                F::from_u128((v1 ^ v2) as u128),
            )
        })
        .collect();
    let xor_rows = (0..NUM_PER_WORD)
        .map(|i| {
            (
                xor_rows[2 * i].0 + xor_rows[2 * i + 1].0 * F::from_u128(8),
                xor_rows[2 * i].1 + xor_rows[2 * i + 1].1 * F::from_u128(8),
            )
        })
        .collect();
    let s_new = convert_bits_to_f(&split_xor_vec);

    (xor_rows, s_new)
}

fn eval_keccak_f_to_bit_vec<F: PrimeField<Repr = [u8; 32]>>(
    value1: F,
    value2: F,
) -> (Vec<(F, F)>, F, F) {
    let v1_vec = convert_field_to_vec_bits(value1);
    let v2_vec = convert_field_to_vec_bits(value2);
    assert_eq!(v1_vec.len(), NUM_BITS_PER_WORD);
    assert_eq!(v2_vec.len(), NUM_BITS_PER_WORD);
    let vec = (0..NUM_PER_WORD)
        .map(|i| {
            (
                F::from_u128(v1_vec[2 * i] as u128 + v1_vec[2 * i + 1] as u128 * 8),
                F::from_u128(v2_vec[2 * i] as u128 + v2_vec[2 * i + 1] as u128 * 8),
            )
        })
        .collect();
    (
        vec,
        F::from_u128(v2_vec[NUM_BITS_PER_WORD - 2] as u128),
        F::from_u128(v2_vec[NUM_BITS_PER_WORD - 1] as u128),
    )
}

fn eval_threes<F: PrimeField<Repr = [u8; 32]>>() -> F {
    let threes_vec: [u8; NUM_BITS_PER_WORD] = [3; NUM_BITS_PER_WORD];
    convert_bits_to_f(&threes_vec.to_vec())
}

fn keccak_xor_table<F: PrimeField + Eq + Hash>(
    ctx: &mut CircuitContext<F, ()>,
    lens: usize,
) -> LookupTable {
    use chiquito::frontend::dsl::cb::*;

    let lookup_xor_row: Queriable<F> = ctx.fixed("xor row");
    let lookup_xor_c: Queriable<F> = ctx.fixed("xor value");

    let constants_value = XOR_VALUE;
    assert_eq!(lens, constants_value.len());
    ctx.pragma_num_steps(lens);
    ctx.fixed_gen(move |ctx| {
        for (i, &value) in constants_value.iter().enumerate().take(lens) {
            ctx.assign(i, lookup_xor_row, F::from(i as u64));
            ctx.assign(i, lookup_xor_c, F::from(value));
        }
    });

    ctx.new_table(table().add(lookup_xor_row).add(lookup_xor_c))
}

fn keccak_chi_table<F: PrimeField + Eq + Hash>(
    ctx: &mut CircuitContext<F, ()>,
    lens: usize,
) -> LookupTable {
    use chiquito::frontend::dsl::cb::*;

    let lookup_chi_row: Queriable<F> = ctx.fixed("chi row");
    let lookup_chi_c: Queriable<F> = ctx.fixed("chi value");

    let constants_value = CHI_VALUE;
    assert_eq!(lens, constants_value.len());
    ctx.pragma_num_steps(lens);
    ctx.fixed_gen(move |ctx| {
        for (i, &value) in constants_value.iter().enumerate().take(lens) {
            ctx.assign(i, lookup_chi_row, F::from(i as u64));
            ctx.assign(i, lookup_chi_c, F::from(value));
        }
    });

    ctx.new_table(table().add(lookup_chi_row).add(lookup_chi_c))
}

fn keccak_round_constants_table<F: PrimeField + Eq + Hash>(
    ctx: &mut CircuitContext<F, ()>,
    lens: usize,
) -> LookupTable {
    use chiquito::frontend::dsl::cb::*;

    let lookup_constant_row: Queriable<F> = ctx.fixed("constant row");
    let lookup_constant_c: Queriable<F> = ctx.fixed("constant value");

    let constants_value = ROUND_CST;
    ctx.pragma_num_steps(lens);
    ctx.fixed_gen(move |ctx| {
        for (i, &value) in constants_value.iter().enumerate().take(lens) {
            ctx.assign(i, lookup_constant_row, F::from(i as u64));
            ctx.assign(i, lookup_constant_c, pack_u64::<F>(value));
        }
    });
    ctx.new_table(table().add(lookup_constant_row).add(lookup_constant_c))
}

struct KeccakFSplitStepArgs<F: PrimeField> {
    absorb_rows: Vec<(F, F, F)>,
    xor_rows: Vec<(F, F)>,
    round_value: F,
}

struct KeccakFThetaSplitStepArgs<F: PrimeField> {
    absorb_rows: Vec<(F, F)>,
    sum_rows: Vec<(F, F)>,
    xor_rows: (Vec<F>, Vec<F>),
    round_value: F,
}

struct KeccakFSumSplitStepArgs<F: PrimeField> {
    absorb_rows: Vec<(F, F)>,
    sum_rows: Vec<(F, F)>,
    xor_rows: (Vec<F>, Vec<F>, F, F),
    round_value: F,
}

struct KeccakFRhoMoveStepArgs<F: PrimeField> {
    absorb_rows: Vec<(F, F)>,
    xor_rows: (Vec<F>, F, F),
    round_value: F,
}

struct KeccakFSplitChiStepArgs<F: PrimeField> {
    absorb_rows: Vec<(F, F, F, F)>,
    xor_rows: Vec<(F, F)>,
    round_value: F,
}

fn keccak_circuit<F: PrimeField<Repr = [u8; 32]> + Eq + Hash>(
    ctx: &mut CircuitContext<F, KeccakCircuit>,
    param: CircuitParams,
) {
    use chiquito::frontend::dsl::cb::*;

    let s_vec: Vec<Vec<Queriable<F>>> = (0..PART_SIZE)
        .map(|i| {
            (0..PART_SIZE)
                .map(|j| ctx.forward(&format!("s[{}][{}]", i, j)))
                .collect()
        })
        .collect();
    let s_new_vec: Vec<Vec<Queriable<F>>> = (0..PART_SIZE)
        .map(|i| {
            (0..PART_SIZE)
                .map(|j| ctx.forward(&format!("s_new[{}][{}]", i, j)))
                .collect()
        })
        .collect();
    let sum_split_value_vec: Vec<Queriable<F>> = (0..PART_SIZE * PART_SIZE)
        .map(|i| ctx.forward(&format!("sum_split_value_{}", i)))
        .collect();
    let sum_sum_split_xor_value_vec: Vec<Queriable<F>> = (0..PART_SIZE)
        .map(|i| ctx.forward(&format!("sum_sum_split_value_{}", i)))
        .collect();
    let sum_sum_split_xor_move_value_vec: Vec<Queriable<F>> = (0..PART_SIZE)
        .map(|i| ctx.forward(&format!("sum_sum_split_move_value_{}", i)))
        .collect();

    let coeff_split_or_absorb_vec: Vec<Queriable<F>> = (0..NUM_PER_WORD)
        .map(|i| ctx.forward(&format!("coeff_split_{}", i)))
        .collect();

    let round: Queriable<F> = ctx.forward("round");
    let coeff64: Queriable<F> = ctx.forward("64");

    let keccak_first_step = ctx.step_type_def("keccak first step", |ctx| {
        let s_vec = s_vec.clone();
        let setup_s_vec = s_vec.clone();

        let absorb_vec = coeff_split_or_absorb_vec.clone();
        let setup_absorb_vec = absorb_vec.clone();

        ctx.setup(move |ctx| {
            for i in 0..PART_SIZE {
                for j in 0..PART_SIZE {
                    ctx.constr(eq(setup_s_vec[i][j], 0));
                    if j * PART_SIZE + i < NUM_WORDS_TO_ABSORB {
                        // xor
                        // 000 xor 000/001 -> 000 + 000/001
                        ctx.transition(eq(
                            setup_s_vec[i][j] + setup_absorb_vec[i * PART_SIZE + j],
                            setup_s_vec[i][j].next(),
                        ));
                    } else {
                        ctx.transition(eq(setup_s_vec[i][j], setup_s_vec[i][j].next()));
                        ctx.constr(eq(setup_absorb_vec[i * PART_SIZE + j], 0));
                    }
                }
            }
            ctx.constr(eq(round, 0));
            ctx.transition(eq(round, round.next()));
        });

        ctx.wg::<(Vec<(F, F)>, F), _>(move |ctx, (absorb_rows, round_value)| {
            for i in 0..PART_SIZE {
                for j in 0..PART_SIZE {
                    ctx.assign(
                        absorb_vec[i * PART_SIZE + j],
                        absorb_rows[i * PART_SIZE + j].1,
                    );
                    ctx.assign(s_vec[i][j], F::ZERO);
                }
            }
            ctx.assign(round, round_value);
        })
    });
    let keccak_pre_step = ctx.step_type_def("keccak pre step", |ctx| {
        let s_vec = s_vec.clone();
        let setup_s_vec = s_vec.clone();

        let s_new_vec = s_new_vec.clone();
        let setup_s_new_vec = s_new_vec.clone();

        let absorb_vec = coeff_split_or_absorb_vec.clone();
        let setup_absorb_vec = absorb_vec.clone();

        let sum_split_value_vec = sum_split_value_vec.clone();
        let setup_sum_split_value_vec = sum_split_value_vec.clone();

        ctx.setup(move |ctx| {
            for i in 0..PART_SIZE {
                for j in 0..PART_SIZE {
                    if j * PART_SIZE + i < NUM_WORDS_TO_ABSORB {
                        // xor
                        ctx.constr(eq(
                            setup_s_vec[i][j] + setup_absorb_vec[i * PART_SIZE + j],
                            setup_sum_split_value_vec[i * PART_SIZE + j],
                        ));
                        ctx.transition(eq(setup_s_new_vec[i][j], setup_s_vec[i][j].next()));
                    } else {
                        ctx.transition(eq(setup_s_vec[i][j], setup_s_vec[i][j].next()));
                        ctx.constr(eq(setup_absorb_vec[i * PART_SIZE + j], 0));
                    }
                }
            }
            ctx.transition(eq(round, round.next()));
        });

        ctx.wg::<(Vec<(F, F, F)>, F), _>(move |ctx, (absorb_rows, round_value)| {
            for i in 0..PART_SIZE {
                for j in 0..PART_SIZE {
                    ctx.assign(
                        sum_split_value_vec[i * PART_SIZE + j],
                        absorb_rows[i * PART_SIZE + j].0 + absorb_rows[i * PART_SIZE + j].1,
                    );
                    ctx.assign(
                        absorb_vec[i * PART_SIZE + j],
                        absorb_rows[i * PART_SIZE + j].1,
                    );
                    ctx.assign(s_vec[i][j], absorb_rows[i * PART_SIZE + j].0);
                    ctx.assign(s_new_vec[i][j], absorb_rows[i * PART_SIZE + j].2);
                }
            }
            ctx.assign(round, round_value);
        })
    });
    let keccak_f_chi_iota_step = ctx.step_type_def("keccak f chi iota step", |ctx| {
        // a[x] = a[x] ^ (~a[x+1] & a[x+2])
        // a[0][0] = a[0][0] ^ round_cst
        let s_vec = s_vec.clone();
        let setup_s_vec: Vec<Vec<Queriable<F>>> = s_vec.clone();

        let s_new_vec = s_new_vec.clone();
        let setup_s_new_vec: Vec<Vec<Queriable<F>>> = s_new_vec.clone();

        let sum_split_value_vec = sum_split_value_vec.clone();
        let setup_sum_split_value_vec = sum_split_value_vec.clone();

        let coeff_split_vec: Vec<Queriable<F>> = coeff_split_or_absorb_vec.clone();
        let setup_coeff_split_vec = coeff_split_vec.clone();

        let split_vec: Vec<Queriable<F>> = (0..NUM_PER_WORD)
            .map(|i| ctx.internal(&format!("split_{}", i)))
            .collect();
        let setup_split_vec = split_vec.clone();

        let split_xor_vec: Vec<Queriable<F>> = (0..NUM_PER_WORD)
            .map(|i| ctx.internal(&format!("split_xor_{}", i)))
            .collect();
        let setup_split_xor_vec = split_xor_vec.clone();

        let threes: Queriable<F> = sum_sum_split_xor_value_vec[0];
        let three2: Queriable<F> = sum_sum_split_xor_value_vec[1];
        let round_cst: Queriable<F> = sum_sum_split_xor_value_vec[2];
        let s_new_add_cst = sum_sum_split_xor_value_vec[3];

        ctx.setup(
            move |ctx: &mut chiquito::frontend::dsl::StepTypeSetupContext<'_, F>| {
                ctx.constr(eq(coeff64, 64));
                ctx.constr(eq(three2, 27));
                let mut sum_split_vec: Expr<F> = three2 * 64 + three2;
                for _ in 2..NUM_PER_WORD {
                    sum_split_vec = sum_split_vec * 64 + three2;
                }
                ctx.constr(eq(sum_split_vec, threes));

                ctx.constr(eq(setup_coeff_split_vec[0], 1));
                for k in 1..NUM_PER_WORD {
                    ctx.constr(eq(
                        setup_coeff_split_vec[k - 1] * coeff64,
                        setup_coeff_split_vec[k],
                    ));
                }
                ctx.add_lookup(param.constants_table.apply(round).apply(round_cst));

                for i in 0..PART_SIZE {
                    for j in 0..PART_SIZE {
                        ctx.constr(eq(
                            threes - setup_s_vec[i][j] - setup_s_vec[i][j]
                                + setup_s_vec[(i + 1) % 5][j]
                                - setup_s_vec[(i + 2) % 5][j],
                            setup_sum_split_value_vec[i * PART_SIZE + j],
                        ));
                    }
                }

                {
                    for k in 0..NUM_PER_WORD {
                        ctx.add_lookup(
                            param
                                .xor_table
                                .apply(setup_split_vec[k])
                                .apply(setup_split_xor_vec[k]),
                        );
                    }
                    // coeff_table
                    let mut sum_s_split_vec: Expr<F> =
                        setup_split_vec[0] * setup_coeff_split_vec[0];
                    let mut sum_s_split_xor_vec: Expr<F> =
                        setup_split_xor_vec[0] * setup_coeff_split_vec[0];
                    for k in 1..NUM_PER_WORD {
                        sum_s_split_vec =
                            sum_s_split_vec + setup_split_vec[k] * setup_coeff_split_vec[k];
                        sum_s_split_xor_vec =
                            sum_s_split_xor_vec + setup_split_xor_vec[k] * setup_coeff_split_vec[k];
                    }
                    ctx.constr(eq(sum_s_split_vec, setup_s_new_vec[0][0] + round_cst));
                    ctx.constr(eq(sum_s_split_xor_vec, s_new_add_cst));
                }

                for i in 0..PART_SIZE {
                    for j in 0..PART_SIZE {
                        if i == 0 && j == 0 {
                            ctx.transition(eq(s_new_add_cst, setup_s_vec[i][j].next()));
                        } else {
                            ctx.transition(eq(setup_s_new_vec[i][j], setup_s_vec[i][j].next()));
                        }
                    }
                }
                ctx.transition(eq((round + 1 - round.next()) * (round + 1 - NUM_ROUNDS), 0));
            },
        );

        ctx.wg::<(Vec<(F, F, F, F)>, Vec<(F, F)>, (F, F, F, F)), _>(
            move |ctx,
                  (
                absorb_rows,
                xor_rows,
                (round_value, round_cst_value, threes_value, s_xor_cst_value),
            )| {
                ctx.assign(coeff64, F::from_u128(64));
                ctx.assign(three2, F::from_u128(27));
                ctx.assign(threes, threes_value);

                let mut coeff_value = F::from_u128(1);
                for &coeff_split in coeff_split_vec.iter().take(NUM_PER_WORD) {
                    ctx.assign(coeff_split, coeff_value);
                    coeff_value *= F::from_u128(64);
                }

                for i in 0..PART_SIZE {
                    for j in 0..PART_SIZE {
                        ctx.assign(s_vec[i][j], absorb_rows[i * PART_SIZE + j].0);
                        ctx.assign(
                            sum_split_value_vec[i * PART_SIZE + j],
                            absorb_rows[i * PART_SIZE + j].1,
                        );
                        ctx.assign(s_new_vec[i][j], absorb_rows[i * PART_SIZE + j].2);
                    }
                }

                for k in 0..NUM_PER_WORD {
                    ctx.assign(split_vec[k], xor_rows[k].0);
                    ctx.assign(split_xor_vec[k], xor_rows[k].1);
                }

                ctx.assign(round, round_value);
                ctx.assign(round_cst, round_cst_value);
                ctx.assign(s_new_add_cst, s_xor_cst_value);
            },
        )
    });
    let keccak_f_split_xor_vec: Vec<StepTypeWGHandler<F, KeccakFSplitStepArgs<F>, _>> = (0
        ..NUM_WORDS_TO_ABSORB)
        .map(|s| {
            
            ctx.step_type_def("keccak_f_split_xor_step", |ctx| {
                let s_vec = s_vec.clone();
                let setup_s_vec = s_vec.clone();

                let s_new_vec = s_new_vec.clone();
                let setup_s_new_vec = s_new_vec.clone();

                let coeff_split_vec: Vec<Queriable<F>> = coeff_split_or_absorb_vec.clone();
                let setup_coeff_split_vec = coeff_split_vec.clone();

                let split_vec: Vec<Queriable<F>> = (0..NUM_PER_WORD)
                    .map(|i| ctx.internal(&format!("split_{}", i)))
                    .collect();
                let setup_split_vec = split_vec.clone();

                let split_xor_vec: Vec<Queriable<F>> = (0..NUM_PER_WORD)
                    .map(|i| ctx.internal(&format!("split_xor_{}", i)))
                    .collect();
                let setup_split_xor_vec = split_xor_vec.clone();

                let sum_split_value_vec = sum_split_value_vec.clone();
                let setup_sum_split_value_vec = sum_split_value_vec.clone();

                ctx.setup(move |ctx| {
                    let s = (s % PART_SIZE) * PART_SIZE + s / PART_SIZE;
                    ctx.constr(eq(setup_coeff_split_vec[0], 1));
                    ctx.constr(eq(coeff64, 64));
                    for k in 1..NUM_PER_WORD {
                        ctx.constr(eq(
                            setup_coeff_split_vec[k - 1] * coeff64,
                            setup_coeff_split_vec[k],
                        ));
                    }

                    for k in 0..NUM_PER_WORD {
                        ctx.add_lookup(
                            param
                                .xor_table
                                .apply(setup_split_vec[k])
                                .apply(setup_split_xor_vec[k]),
                        );
                    }

                    let mut sum_split_vec: Expr<F> = setup_split_vec[0] * setup_coeff_split_vec[0];
                    let mut sum_split_xor_vec: Expr<F> =
                        setup_split_xor_vec[0] * setup_coeff_split_vec[0];
                    for k in 1..NUM_PER_WORD {
                        sum_split_vec =
                            sum_split_vec + setup_split_vec[k] * setup_coeff_split_vec[k];
                        sum_split_xor_vec =
                            sum_split_xor_vec + setup_split_xor_vec[k] * setup_coeff_split_vec[k];
                    }
                    ctx.constr(eq(sum_split_vec, setup_sum_split_value_vec[s]));
                    ctx.constr(eq(
                        sum_split_xor_vec,
                        setup_s_new_vec[s / PART_SIZE][s % PART_SIZE],
                    ));
                    for &setup_sum_split_value in
                        setup_sum_split_value_vec.iter().take(PART_SIZE * PART_SIZE)
                    {
                        ctx.transition(eq(setup_sum_split_value, setup_sum_split_value.next()));
                    }
                    for i in 0..PART_SIZE {
                        for j in 0..PART_SIZE {
                            ctx.transition(eq(setup_s_vec[i][j], setup_s_vec[i][j].next()));
                            ctx.transition(eq(setup_s_new_vec[i][j], setup_s_new_vec[i][j].next()));
                        }
                    }
                    ctx.transition(eq(round, round.next()));
                });

                ctx.wg::<KeccakFSplitStepArgs<F>, _>(move |ctx, values| {
                    ctx.assign(coeff64, F::from_u128(64));
                    let mut coeff_value = F::from_u128(1);
                    for &coeff_split in coeff_split_vec.iter().take(NUM_PER_WORD) {
                        ctx.assign(coeff_split, coeff_value);
                        coeff_value *= F::from_u128(64)
                    }
                    for (k, &sum_split_value) in sum_split_value_vec
                        .iter()
                        .enumerate()
                        .take(PART_SIZE * PART_SIZE)
                    {
                        ctx.assign(sum_split_value, values.absorb_rows[k].1);
                    }

                    for i in 0..NUM_PER_WORD {
                        ctx.assign(split_vec[i], values.xor_rows[i].0);
                        ctx.assign(split_xor_vec[i], values.xor_rows[i].1);
                    }
                    for i in 0..PART_SIZE {
                        for j in 0..PART_SIZE {
                            ctx.assign(s_vec[i][j], values.absorb_rows[i * PART_SIZE + j].0);
                            ctx.assign(s_new_vec[i][j], values.absorb_rows[i * PART_SIZE + j].2);
                        }
                    }
                    ctx.assign(round, values.round_value);
                })
            })
        })
        .collect();

    let keccak_f_theta_split_xor_step_vec: Vec<
        StepTypeWGHandler<F, KeccakFSumSplitStepArgs<F>, _>,
    > = (0..PART_SIZE)
        .map(|s| {
            ctx.step_type_def("keccak_f_sum_split_xor_step", |ctx| {
                let s_vec = s_vec.clone();
                let setup_s_vec = s_vec.clone();

                let coeff_split_vec: Vec<Queriable<F>> = coeff_split_or_absorb_vec.clone();
                let setup_coeff_split_vec = coeff_split_vec.clone();

                let split_vec: Vec<Queriable<F>> = (0..NUM_PER_WORD)
                    .map(|i| ctx.internal(&format!("split_{}", i)))
                    .collect();
                let setup_split_vec = split_vec.clone();

                let split_xor_vec: Vec<Queriable<F>> = (0..NUM_PER_WORD)
                    .map(|i| ctx.internal(&format!("split_xor_{}", i)))
                    .collect();
                let setup_split_xor_vec = split_xor_vec.clone();

                let sum_sum_split_xor_value_vec = sum_sum_split_xor_value_vec.clone();
                let setup_sum_sum_split_xor_value_vec = sum_sum_split_xor_value_vec.clone();

                let sum_sum_split_xor_move_value_vec = sum_sum_split_xor_move_value_vec.clone();
                let setup_sum_sum_split_xor_move_value_vec =
                    sum_sum_split_xor_move_value_vec.clone();

                let bit_0 = s_new_vec[0][0];
                let bit_1 = s_new_vec[0][1];

                ctx.setup(move |ctx| {
                    ctx.constr(eq(setup_coeff_split_vec[0], 1));
                    ctx.constr(eq(coeff64, 64));
                    for k in 1..NUM_PER_WORD {
                        ctx.constr(eq(
                            setup_coeff_split_vec[k - 1] * coeff64,
                            setup_coeff_split_vec[k],
                        ));
                    }

                    for k in 0..NUM_PER_WORD {
                        ctx.add_lookup(
                            param
                                .xor_table
                                .apply(setup_split_vec[k])
                                .apply(setup_split_xor_vec[k]),
                        );
                    }

                    let mut sum_split_vec: Expr<F> = setup_split_vec[0] * setup_coeff_split_vec[0];
                    let mut sum_split_xor_vec: Expr<F> =
                        setup_split_xor_vec[0] * setup_coeff_split_vec[0];
                    for k in 1..NUM_PER_WORD {
                        sum_split_vec =
                            sum_split_vec + setup_split_vec[k] * setup_coeff_split_vec[k];
                        sum_split_xor_vec =
                            sum_split_xor_vec + setup_split_xor_vec[k] * setup_coeff_split_vec[k];
                    }

                    ctx.constr(eq(
                        setup_s_vec[s][0]
                            + setup_s_vec[s][1]
                            + setup_s_vec[s][2]
                            + setup_s_vec[s][3]
                            + setup_s_vec[s][4],
                        sum_split_vec,
                    ));

                    ctx.constr(eq(
                        sum_split_xor_vec.clone(),
                        setup_sum_sum_split_xor_value_vec[s],
                    ));
                    ctx.constr(eq(bit_0 * (bit_0 - 1), 0));
                    ctx.constr(eq(bit_1 * (bit_1 - 1), 0));
                    ctx.constr(eq(bit_0 + bit_1 * 8, setup_split_xor_vec[NUM_PER_WORD - 1]));
                    ctx.constr(eq(
                        (sum_split_xor_vec - bit_1 * setup_coeff_split_vec[NUM_PER_WORD - 1] * 8)
                            * 8
                            + bit_1,
                        setup_sum_sum_split_xor_move_value_vec[s],
                    ));

                    for k in 0..setup_sum_sum_split_xor_value_vec.len() {
                        ctx.transition(eq(
                            setup_sum_sum_split_xor_value_vec[k],
                            setup_sum_sum_split_xor_value_vec[k].next(),
                        ));
                        ctx.transition(eq(
                            setup_sum_sum_split_xor_move_value_vec[k],
                            setup_sum_sum_split_xor_move_value_vec[k].next(),
                        ));
                    }
                    for setup_s_vec in setup_s_vec.iter().take(PART_SIZE) {
                        for &setup_s in setup_s_vec.iter().take(PART_SIZE) {
                            ctx.transition(eq(setup_s, setup_s.next()));
                        }
                    }
                    ctx.transition(eq(round, round.next()));
                });

                ctx.wg::<KeccakFSumSplitStepArgs<F>, _>(move |ctx, values| {
                    ctx.assign(coeff64, F::from_u128(64));
                    let mut coeff_value = F::from_u128(1);
                    for &coeff_split in coeff_split_vec.iter().take(NUM_PER_WORD) {
                        ctx.assign(coeff_split, coeff_value);
                        coeff_value *= F::from_u128(64)
                    }
                    for k in 0..sum_sum_split_xor_value_vec.len() {
                        ctx.assign(sum_sum_split_xor_value_vec[k], values.sum_rows[k].0);
                        ctx.assign(sum_sum_split_xor_move_value_vec[k], values.sum_rows[k].1);
                    }

                    for i in 0..NUM_PER_WORD {
                        ctx.assign(split_vec[i], values.xor_rows.0[i]);
                        ctx.assign(split_xor_vec[i], values.xor_rows.1[i]);
                    }

                    for k in 0..PART_SIZE * PART_SIZE {
                        ctx.assign(s_vec[k / PART_SIZE][k % PART_SIZE], values.absorb_rows[k].0);
                    }
                    ctx.assign(round, values.round_value);
                    ctx.assign(bit_0, values.xor_rows.2);
                    ctx.assign(bit_1, values.xor_rows.3);
                })
            })
        })
        .collect();

    let keccak_f_theta_sum_xor_step_vec: Vec<
        StepTypeWGHandler<F, KeccakFThetaSplitStepArgs<F>, _>,
    > = (0..PART_SIZE * PART_SIZE)
        .map(|s| {
            ctx.step_type_def("keccak_f_theta_sum_xor_step", |ctx| {
                let s_vec = s_vec.clone();
                let setup_s_vec = s_vec.clone();

                let s_new_vec = s_new_vec.clone();
                let setup_s_new_vec = s_new_vec.clone();

                let coeff_split_vec: Vec<Queriable<F>> = coeff_split_or_absorb_vec.clone();
                let setup_coeff_split_vec = coeff_split_vec.clone();

                let split_vec: Vec<Queriable<F>> = (0..NUM_PER_WORD)
                    .map(|i| ctx.internal(&format!("split_{}", i)))
                    .collect();
                let setup_split_vec = split_vec.clone();

                let split_xor_vec: Vec<Queriable<F>> = (0..NUM_PER_WORD)
                    .map(|i| ctx.internal(&format!("split_xor_{}", i)))
                    .collect();
                let setup_split_xor_vec = split_xor_vec.clone();

                let sum_sum_split_xor_value_vec = sum_sum_split_xor_value_vec.clone();
                let setup_sum_sum_split_xor_value_vec = sum_sum_split_xor_value_vec.clone();

                let sum_sum_split_xor_move_value_vec = sum_sum_split_xor_move_value_vec.clone();
                let setup_sum_sum_split_xor_move_value_vec =
                    sum_sum_split_xor_move_value_vec.clone();

                ctx.setup(move |ctx| {
                    ctx.constr(eq(setup_coeff_split_vec[0], 1));
                    ctx.constr(eq(coeff64, 64));
                    for k in 1..NUM_PER_WORD {
                        ctx.constr(eq(
                            setup_coeff_split_vec[k - 1] * coeff64,
                            setup_coeff_split_vec[k],
                        ));
                    }

                    for k in 0..NUM_PER_WORD {
                        ctx.add_lookup(
                            param
                                .xor_table
                                .apply(setup_split_vec[k])
                                .apply(setup_split_xor_vec[k]),
                        );
                    }

                    let mut sum_split_vec: Expr<F> = setup_split_vec[0] * setup_coeff_split_vec[0];
                    let mut sum_split_xor_vec: Expr<F> =
                        setup_split_xor_vec[0] * setup_coeff_split_vec[0];
                    for k in 1..NUM_PER_WORD {
                        sum_split_vec =
                            sum_split_vec + setup_split_vec[k] * setup_coeff_split_vec[k];
                        sum_split_xor_vec =
                            sum_split_xor_vec + setup_split_xor_vec[k] * setup_coeff_split_vec[k];
                    }
                    let i = s / PART_SIZE;
                    let j = s % PART_SIZE;
                    ctx.constr(eq(
                        sum_split_vec,
                        setup_s_vec[i][j]
                            + setup_sum_sum_split_xor_value_vec[(i + PART_SIZE - 1) % PART_SIZE]
                            + setup_sum_sum_split_xor_move_value_vec[(i + 1) % PART_SIZE],
                    ));
                    ctx.constr(eq(
                        sum_split_xor_vec,
                        setup_s_new_vec[s / PART_SIZE][s % PART_SIZE],
                    ));
                    if s < PART_SIZE * PART_SIZE - 1 {
                        for i in 0..PART_SIZE {
                            for j in 0..PART_SIZE {
                                ctx.transition(eq(setup_s_vec[i][j], setup_s_vec[i][j].next()));
                                ctx.transition(eq(
                                    setup_s_new_vec[i][j],
                                    setup_s_new_vec[i][j].next(),
                                ));
                            }
                        }
                        for &setup_sum_sum_split_xor_value in &setup_sum_sum_split_xor_value_vec {
                            ctx.transition(eq(
                                setup_sum_sum_split_xor_value,
                                setup_sum_sum_split_xor_value.next(),
                            ));
                        }
                        for &setup_sum_sum_split_xor_move_value in
                            &setup_sum_sum_split_xor_move_value_vec
                        {
                            ctx.transition(eq(
                                setup_sum_sum_split_xor_move_value,
                                setup_sum_sum_split_xor_move_value.next(),
                            ));
                        }
                    } else {
                        for i in 0..PART_SIZE {
                            for j in 0..PART_SIZE {
                                ctx.transition(eq(setup_s_new_vec[i][j], setup_s_vec[i][j].next()));
                            }
                        }
                    }
                    ctx.transition(eq(round, round.next()));
                });

                ctx.wg::<KeccakFThetaSplitStepArgs<F>, _>(move |ctx, values| {
                    ctx.assign(coeff64, F::from_u128(64));
                    let mut coeff_value = F::from_u128(1);
                    for &coeff_split in coeff_split_vec.iter().take(NUM_PER_WORD) {
                        ctx.assign(coeff_split, coeff_value);
                        coeff_value *= F::from_u128(64)
                    }

                    for k in 0..sum_sum_split_xor_value_vec.len() {
                        ctx.assign(sum_sum_split_xor_value_vec[k], values.sum_rows[k].0);
                        ctx.assign(sum_sum_split_xor_move_value_vec[k], values.sum_rows[k].1);
                    }

                    for i in 0..NUM_PER_WORD {
                        ctx.assign(split_vec[i], values.xor_rows.0[i]);
                        ctx.assign(split_xor_vec[i], values.xor_rows.1[i]);
                    }
                    for i in 0..PART_SIZE {
                        for j in 0..PART_SIZE {
                            ctx.assign(s_vec[i][j], values.absorb_rows[i * PART_SIZE + j].0);
                            ctx.assign(s_new_vec[i][j], values.absorb_rows[i * PART_SIZE + j].1);
                        }
                    }
                    ctx.assign(round, values.round_value);
                })
            })
        })
        .collect();

    let keccak_f_rho_move_and_pi_step_vec: Vec<StepTypeWGHandler<F, KeccakFRhoMoveStepArgs<F>, _>>= (0..PART_SIZE * PART_SIZE).map(|s|
        // a[x][y][z] = a[x][y][z-(t+1)(t+2)/2]
        ctx.step_type_def("keccak_f_rho_move_and_pi_step", |ctx|{
            let s_vec = s_vec.clone();
            let setup_s_vec = s_vec.clone();

            let s_new_vec = s_new_vec.clone();
            let setup_s_new_vec:  Vec<Vec<Queriable<F>>> =  s_new_vec.clone();

            let coeff_split_vec: Vec<Queriable<F>> = coeff_split_or_absorb_vec.clone();
            let setup_coeff_split_vec = coeff_split_vec.clone();

            let split_vec: Vec<Queriable<F>>= (0..NUM_PER_WORD).map(|i|ctx.internal(&format!("split_{}", i))).collect();
            let setup_split_vec = split_vec.clone();

            // let bit_0 = ctx.internal("bit0");
            // let bit_1 = ctx.internal("bit1");
            let bit_0 = sum_sum_split_xor_value_vec[0];
            let bit_1 = sum_sum_split_xor_value_vec[1];


            ctx.setup(move |ctx| {
                let mut i = 0;
                let mut j = 0;
                for t in 0..s {
                    if t == 0 {
                        i = 1;
                        j = 0;
                    } else {
                        let m = j;
                        j = (2*i+3*j)%5;
                        i = m;
                    }
                }
                let v = ((s + 1) * s / 2) % NUM_BITS_PER_WORD;

                if v == 0 {
                    ctx.constr(eq(setup_s_vec[i][j], setup_s_new_vec[i][j]));
                } else {

                    ctx.constr(eq(setup_coeff_split_vec[0], 1));
                    ctx.constr(eq(coeff64, 64));
                    for k in 1..NUM_PER_WORD {
                        ctx.constr(eq(setup_coeff_split_vec[k-1] * coeff64, setup_coeff_split_vec[k]));
                    }

                    let mut sum_split_vec: Expr<F> = setup_split_vec[0] * setup_coeff_split_vec[0];
                    for k in 1..NUM_PER_WORD {
                        sum_split_vec = sum_split_vec + setup_split_vec[k] * setup_coeff_split_vec[k];
                    }

                    let mut sum_split_move_vec: Expr<F> = setup_split_vec[(NUM_PER_WORD - v/2) % NUM_PER_WORD] * setup_coeff_split_vec[0];
                    for k in 1..NUM_PER_WORD {
                        sum_split_move_vec = sum_split_move_vec + setup_split_vec[(k + NUM_PER_WORD - v/2) % NUM_PER_WORD] * setup_coeff_split_vec[k];
                    }
                    if v % 2 != 0  {
                        sum_split_move_vec = bit_1 + bit_0 * setup_coeff_split_vec[NUM_PER_WORD - 1] * 8;
                        for k in 1..NUM_PER_WORD {
                            sum_split_move_vec = sum_split_move_vec + setup_split_vec[(k - 1 + NUM_PER_WORD - (v - 1)/2) % NUM_PER_WORD] * setup_coeff_split_vec[k-1] * 8;
                        }
                        ctx.constr(eq(bit_0 * (bit_0 - 1), 0));
                        ctx.constr(eq(bit_1 * (bit_1 - 1), 0));
                        ctx.constr(eq(bit_0 + bit_1 * 8, setup_split_vec[(NUM_PER_WORD - (v + 1)/2) % NUM_PER_WORD]));
                    }
                    ctx.constr(eq(sum_split_vec, setup_s_vec[i][j]));
                    ctx.constr(eq(sum_split_move_vec, setup_s_new_vec[i][j]));
                }

                if s == PART_SIZE * PART_SIZE - 1 {
                    for k in 0..PART_SIZE * PART_SIZE {
                        let i = k / PART_SIZE;
                        let j = k % PART_SIZE;
                        let x = j;
                        let y = (2 * i + 3 * j) % 5;
                        ctx.transition(eq(setup_s_new_vec[i][j], setup_s_vec[x][y].next()));
                    }
                } else {
                    for i in 0..PART_SIZE {
                        for j in 0..PART_SIZE {
                            ctx.transition(eq(setup_s_vec[i][j], setup_s_vec[i][j].next()));
                            ctx.transition(eq(setup_s_new_vec[i][j], setup_s_new_vec[i][j].next()));
                        }
                    }
                }
                ctx.transition(eq(round, round.next()));
            });
            ctx.wg::<KeccakFRhoMoveStepArgs<F>, _>(move |ctx, values|{
                ctx.assign(coeff64, F::from_u128(64));
                let mut coeff_value = F::from_u128(1);
                for &coeff_split in coeff_split_vec.iter().take(NUM_BITS_PER_WORD) {
                    ctx.assign(coeff_split, coeff_value);
                    coeff_value *= F::from_u128(64)
                }
                for i in 0..PART_SIZE {
                    for j in 0..PART_SIZE {
                        ctx.assign(s_vec[i][j], values.absorb_rows[i * PART_SIZE + j].0);
                        ctx.assign(s_new_vec[i][j], values.absorb_rows[i * PART_SIZE + j].1);
                    }
                }
                for (i, &split) in split_vec.iter().enumerate().take(NUM_BITS_PER_WORD) {
                    ctx.assign(split, values.xor_rows.0[i]);
                }
                ctx.assign(round, values.round_value);
                ctx.assign(bit_0, values.xor_rows.1);
                ctx.assign(bit_1, values.xor_rows.2);
            })
        })
    ).collect();

    let keccak_f_split_chi_vec: Vec<StepTypeWGHandler<F, KeccakFSplitChiStepArgs<F>, _>>= (0..PART_SIZE * PART_SIZE).map(|s|
        // a[x] = a[x] ^ (~a[x+1] & a[x+2])
        ctx.step_type_def("keccak_f_split_chi_step", |ctx|{
            let s_vec = s_vec.clone();
            let setup_s_vec = s_vec.clone();

            let s_new_vec = s_new_vec.clone();
            let setup_s_new_vec: Vec<Vec<Queriable<F>>> =  s_new_vec.clone();

            let coeff_split_vec: Vec<Queriable<F>> = coeff_split_or_absorb_vec.clone(); //(0..NUM_BITS_PER_WORD).map(|i|ctx.internal(&format!("coeff_split_{}", i))).collect();
            let setup_coeff_split_vec = coeff_split_vec.clone();

            let split_vec: Vec<Queriable<F>>= (0..NUM_PER_WORD).map(|i|ctx.internal(&format!("split_{}", i))).collect();
            let setup_split_vec = split_vec.clone();

            let split_chi_vec: Vec<Queriable<F>>= (0..NUM_PER_WORD).map(|i|ctx.internal(&format!("split_chi_{}", i))).collect();
            let setup_split_chi_vec = split_chi_vec.clone();

            let sum_split_value_vec = sum_split_value_vec.clone();
            let setup_sum_split_value_vec = sum_split_value_vec.clone();

            ctx.setup(move |ctx| {
                ctx.constr(eq(setup_coeff_split_vec[0], 1));
                ctx.constr(eq(coeff64, 64
                ));
                for k in 1..NUM_PER_WORD {
                    ctx.constr(eq(setup_coeff_split_vec[k-1] * coeff64, setup_coeff_split_vec[k]));
                }

                for k in 0..NUM_PER_WORD{
                    ctx.add_lookup(param.chi_table.apply(setup_split_vec[k]).apply(setup_split_chi_vec[k]));
                }

                let mut sum_split_vec: Expr<F> = setup_split_vec[0] * setup_coeff_split_vec[0];
                let mut sum_split_chi_vec: Expr<F> = setup_split_chi_vec[0] * setup_coeff_split_vec[0];
                for k in 1..NUM_PER_WORD {
                    sum_split_vec = sum_split_vec + setup_split_vec[k] * setup_coeff_split_vec[k];
                    sum_split_chi_vec = sum_split_chi_vec + setup_split_chi_vec[k] * setup_coeff_split_vec[k];
                }
                ctx.constr(eq(sum_split_vec, setup_sum_split_value_vec[s]));
                ctx.constr(eq(sum_split_chi_vec, setup_s_new_vec[s / PART_SIZE][s % PART_SIZE]));
                for i in 0..PART_SIZE {
                    for j in 0..PART_SIZE {
                        ctx.transition(eq(setup_sum_split_value_vec[i * PART_SIZE + j], setup_sum_split_value_vec[i * PART_SIZE + j].next()));
                        ctx.transition(eq(setup_s_vec[i][j], setup_s_vec[i][j].next()));
                        ctx.transition(eq(setup_s_new_vec[i][j], setup_s_new_vec[i][j].next()));
                    }
                }
                ctx.transition(eq(round, round.next()));
            });
            ctx.wg::<KeccakFSplitChiStepArgs<F>, _>(move |ctx, values|{
                ctx.assign(coeff64, F::from_u128(64));
                let mut coeff_value = F::from_u128(1);
                for &coeff_split in coeff_split_vec.iter().take(NUM_PER_WORD) {
                    ctx.assign(coeff_split, coeff_value);
                    coeff_value *= F::from_u128(64)
                }

                for i in 0..NUM_PER_WORD {
                    ctx.assign(split_vec[i], values.xor_rows[i].0);
                    ctx.assign(split_chi_vec[i], values.xor_rows[i].1);
                }
                for i in 0..PART_SIZE {
                    for j in 0..PART_SIZE {
                        ctx.assign(s_vec[i][j], values.absorb_rows[i * PART_SIZE + j].0);
                        ctx.assign(sum_split_value_vec[i * PART_SIZE + j], values.absorb_rows[i * PART_SIZE + j].1);
                        ctx.assign(s_new_vec[i][j], values.absorb_rows[i * PART_SIZE + j].2);
                    }
                }
                ctx.assign(round, values.round_value);
            })
        })
    ).collect();

    ctx.pragma_first_step(&keccak_first_step);
    ctx.pragma_last_step(&keccak_f_chi_iota_step);

    ctx.pragma_num_steps(param.step_num);

    ctx.trace(move |ctx, params| {
        let mut bits = params.bits.clone();
        println!("intput bits(without padding) = {:?}", bits);
        // padding
        bits.push(1);
        while (bits.len() + 1) % RATE_IN_BITS != 0 {
            bits.push(0);
        }
        bits.push(1);
        println!("intput bits(with padding) = {:?}", bits);

        let mut s_new = [[F::ZERO; PART_SIZE]; PART_SIZE];

        // chunks
        let chunks = bits.chunks(RATE_IN_BITS);

        // absorb
        for (k, chunk) in chunks.enumerate() {
            let s: Vec<Vec<F>> = s_new.iter().map(|s_new| s_new.to_vec()).collect();
            let absorbs: Vec<F> = (0..PART_SIZE * PART_SIZE)
                .map(|idx| {
                    let i = idx % PART_SIZE;
                    let j = idx / PART_SIZE;
                    let mut absorb = F::ZERO;
                    if idx < NUM_WORDS_TO_ABSORB {
                        absorb = pack(&chunk[idx * 64..(idx + 1) * 64]);
                        s_new[i][j] = field_xor(s[i][j], absorb);
                    } else {
                        s_new[i][j] = s[i][j];
                    }
                    absorb
                })
                .collect();

            if k == 0 {
                let absorb_rows: Vec<(F, F)> = (0..PART_SIZE * PART_SIZE)
                    .map(|idx| {
                        (
                            s[idx / PART_SIZE][idx % PART_SIZE],
                            absorbs[(idx % PART_SIZE) * PART_SIZE + (idx / PART_SIZE)],
                        )
                    })
                    .collect();
                ctx.add(&keccak_first_step, (absorb_rows, F::ZERO));
            } else {
                let absorb_rows: Vec<(F, F, F)> = (0..PART_SIZE * PART_SIZE)
                    .map(|idx| {
                        (
                            s[idx / PART_SIZE][idx % PART_SIZE],
                            absorbs[(idx % PART_SIZE) * PART_SIZE + (idx / PART_SIZE)],
                            s_new[idx / PART_SIZE][idx % PART_SIZE],
                        )
                    })
                    .collect();
                let sum_rows: Vec<(F, F, F)> = (0..PART_SIZE * PART_SIZE)
                    .map(|i| {
                        let sum = absorb_rows[i].0 + absorb_rows[i].1;
                        (absorb_rows[i].0, sum, absorb_rows[i].2)
                    })
                    .collect();
                for t in 0..NUM_WORDS_TO_ABSORB {
                    let i = t % PART_SIZE;
                    let j = t / PART_SIZE;
                    let (xor_rows, _, _) = eval_keccak_f_to_bit_vec::<F>(
                        sum_rows[i * PART_SIZE + j].1,
                        sum_rows[i * PART_SIZE + j].2,
                    );
                    ctx.add(
                        &keccak_f_split_xor_vec[t],
                        KeccakFSplitStepArgs {
                            absorb_rows: sum_rows.clone(),
                            xor_rows,
                            round_value: F::ZERO,
                        },
                    );
                }
                ctx.add(&keccak_pre_step, (absorb_rows, F::ZERO));
            }

            for (round, &cst) in ROUND_CST.iter().enumerate().take(NUM_ROUNDS) {
                // Theta
                let theta_values = eval_keccak_f_theta_step::<F>(&mut s_new);
                for (i, keccak_f_theta_split_xor_step) in keccak_f_theta_split_xor_step_vec
                    .iter()
                    .enumerate()
                    .take(PART_SIZE)
                {
                    ctx.add(
                        keccak_f_theta_split_xor_step,
                        KeccakFSumSplitStepArgs {
                            absorb_rows: theta_values.value_rows.clone(),
                            xor_rows: theta_values.sum_xor_vec[i].clone(),
                            sum_rows: theta_values.sum_row_vec.clone(),
                            round_value: F::from(round as u64),
                        },
                    );
                }

                for (i, keccak_f_split_xor) in keccak_f_theta_sum_xor_step_vec
                    .iter()
                    .enumerate()
                    .take(PART_SIZE * PART_SIZE)
                {
                    ctx.add(
                        keccak_f_split_xor,
                        KeccakFThetaSplitStepArgs {
                            absorb_rows: theta_values.value_rows.clone(),
                            sum_rows: theta_values.sum_row_vec.clone(),
                            xor_rows: theta_values.xor_vec[i].clone(),
                            round_value: F::from(round as u64),
                        },
                    );
                }

                // rho
                let values = eval_keccak_f_rho_step::<F>(&mut s_new);
                for (s, keccak_f_rho_move_and_pi_step) in keccak_f_rho_move_and_pi_step_vec
                    .iter()
                    .enumerate()
                    .take(PART_SIZE * PART_SIZE)
                {
                    ctx.add(
                        keccak_f_rho_move_and_pi_step,
                        KeccakFRhoMoveStepArgs {
                            absorb_rows: values.absorb_rows.clone(),
                            xor_rows: values.xor_rows[s].clone(),
                            round_value: F::from(round as u64),
                        },
                    );
                }

                // pi
                eval_keccak_f_pi_step::<F>(&mut s_new);

                // chi
                let absorb_rows: Vec<(F, F, F, F)> = eval_keccak_f_chi_step::<F>(&mut s_new);
                for i in 0..PART_SIZE * PART_SIZE {
                    let (xor_rows, _, _) =
                        eval_keccak_f_to_bit_vec::<F>(absorb_rows[i].1, absorb_rows[i].2);
                    ctx.add(
                        &keccak_f_split_chi_vec[i],
                        KeccakFSplitChiStepArgs {
                            absorb_rows: absorb_rows.clone(),
                            xor_rows,
                            round_value: F::from(round as u64),
                        },
                    );
                }
                let (xor_rows, s_new_value) = eval_keccak_f_iota_step::<F>(s_new[0][0], cst);
                ctx.add(
                    &keccak_f_chi_iota_step,
                    (
                        absorb_rows,
                        xor_rows,
                        (
                            F::from(round as u64),
                            pack_u64::<F>(cst),
                            eval_threes(),
                            s_new_value,
                        ),
                    ),
                );
                s_new[0][0] = s_new_value;
            }
        }

        // squeezing
        let mut output: Vec<F> = (0..NUM_WORDS_TO_ABSORB)
            .map(|k| {
                let i = k % PART_SIZE;
                let j = k / PART_SIZE;
                s_new[i][j]
            })
            .collect();

        for _ in 0..(params.output_len - 1) / RATE_IN_BITS {
            for (round, &cst) in ROUND_CST.iter().enumerate().take(NUM_ROUNDS) {
                // Theta
                // a[x][y][z] = a[x][y][z] + \sum_y' a[x-1][y'][z] + \sum a[x+1][y'][z-1]
                let theta_values = eval_keccak_f_theta_step::<F>(&mut s_new);

                for (i, keccak_f_theta_split_xor_step) in keccak_f_theta_split_xor_step_vec
                    .iter()
                    .enumerate()
                    .take(PART_SIZE)
                {
                    ctx.add(
                        keccak_f_theta_split_xor_step,
                        KeccakFSumSplitStepArgs {
                            absorb_rows: theta_values.value_rows.clone(),
                            xor_rows: theta_values.sum_xor_vec[i].clone(),
                            sum_rows: theta_values.sum_row_vec.clone(),
                            round_value: F::from(round as u64),
                        },
                    );
                }

                for (i, keccak_f_split_xor) in keccak_f_theta_sum_xor_step_vec
                    .iter()
                    .enumerate()
                    .take(PART_SIZE * PART_SIZE)
                {
                    ctx.add(
                        keccak_f_split_xor,
                        KeccakFThetaSplitStepArgs {
                            absorb_rows: theta_values.value_rows.clone(),
                            sum_rows: theta_values.sum_row_vec.clone(),
                            xor_rows: theta_values.xor_vec[i].clone(),
                            round_value: F::from(round as u64),
                        },
                    );
                }

                // rho
                // a[x][y][z] = a[x][y][z-(t+1)(t+2)/2]
                let values = eval_keccak_f_rho_step::<F>(&mut s_new);
                for (s, keccak_f_rho_move_and_pi_step) in keccak_f_rho_move_and_pi_step_vec
                    .iter()
                    .enumerate()
                    .take(PART_SIZE * PART_SIZE)
                {
                    ctx.add(
                        keccak_f_rho_move_and_pi_step,
                        KeccakFRhoMoveStepArgs {
                            absorb_rows: values.absorb_rows.clone(),
                            xor_rows: values.xor_rows[s].clone(),
                            round_value: F::from(round as u64),
                        },
                    );
                }

                // pi
                eval_keccak_f_pi_step::<F>(&mut s_new);

                // chi
                // a[x] = a[x] ^ (~a[x+1] & a[x+2])
                let absorb_rows = eval_keccak_f_chi_step::<F>(&mut s_new);
                for i in 0..PART_SIZE * PART_SIZE {
                    let (xor_rows, _, _) =
                        eval_keccak_f_to_bit_vec::<F>(absorb_rows[i].1, absorb_rows[i].2);
                    ctx.add(
                        &keccak_f_split_chi_vec[i],
                        KeccakFSplitChiStepArgs {
                            absorb_rows: absorb_rows.clone(),
                            xor_rows,
                            round_value: F::from(round as u64),
                        },
                    );
                }
                let (xor_rows, s_new_value) = eval_keccak_f_iota_step::<F>(s_new[0][0], cst);
                ctx.add(
                    &keccak_f_chi_iota_step,
                    (
                        absorb_rows,
                        xor_rows,
                        (
                            F::from(round as u64),
                            pack_u64::<F>(cst),
                            eval_threes(),
                            s_new_value,
                        ),
                    ),
                );
                s_new[0][0] = s_new_value;
            }
            let mut z_vec: Vec<F> = (0..NUM_WORDS_TO_ABSORB)
                .map(|k| s_new[k % PART_SIZE][k / PART_SIZE])
                .collect();
            output.append(&mut z_vec);
        }

        let mut output_bits: Vec<u8> = Vec::new();
        for out in output {
            let mut out_bits = convert_field_to_vec_bits(out);
            output_bits.append(&mut out_bits);
        }
        assert!(output_bits.len() >= params.output_len);
        output_bits = output_bits[0..params.output_len].to_vec();
        println!("output = {:?}", output_bits);
        // println!("num_steps = {:?}", ctx.num_steps);
    });
}

#[derive(Default)]
struct KeccakCircuit {
    pub bits: Vec<u8>,
    pub output_len: usize,
}

struct CircuitParams {
    pub constants_table: LookupTable,
    pub xor_table: LookupTable,
    pub chi_table: LookupTable,
    pub step_num: usize,
}

fn keccak_super_circuit<F: PrimeField<Repr = [u8; 32]> + Eq + Hash>(
    input_len: usize,
    output_len: usize,
) -> SuperCircuit<F, KeccakCircuit> {
    super_circuit::<F, KeccakCircuit, _>("keccak", |ctx| {
        let in_n = (input_len + 1 + RATE_IN_BITS) / RATE_IN_BITS;
        let out_n = output_len / RATE_IN_BITS;
        let step_num = in_n
            * (1 + NUM_WORDS_TO_ABSORB + (NUM_ROUNDS) * ((25 + 5) + (25) + (25 + 1)))
            - NUM_WORDS_TO_ABSORB
            + out_n * ((NUM_ROUNDS) * ((25 + 5) + (25) + (25 + 1)));
        // println!("step_num = {}", step_num);
        let single_config = config(SingleRowCellManager {}, SimpleStepSelectorBuilder {});
        let (_, constants_table) = ctx.sub_circuit(
            single_config.clone(),
            keccak_round_constants_table,
            NUM_ROUNDS + 1,
        );
        let (_, xor_table) =
            ctx.sub_circuit(single_config.clone(), keccak_xor_table, NUM_BITS_PER_WORD);
        let (_, chi_table) = ctx.sub_circuit(single_config, keccak_chi_table, NUM_BITS_PER_WORD);

        let params = CircuitParams {
            constants_table,
            xor_table,
            chi_table,
            step_num,
        };

        let maxwidth_config = config(
            MaxWidthCellManager::new(500, true),
            SimpleStepSelectorBuilder {},
        );
        let (keccak, _) = ctx.sub_circuit(maxwidth_config, keccak_circuit, params);

        ctx.mapping(move |ctx, values| {
            ctx.map(&keccak, values);
        })
    })
}

use chiquito::plonkish::ir::Circuit;
use chiquito::plonkish::backend::plaf::chiquito2Plaf;
use polyexen::plaf::{Witness, Plaf, PlafDisplayBaseTOML, PlafDisplayFixedCSV, backends::halo2::PlafH2Circuit, WitnessDisplayCSV};

fn write_files(name: &str, plaf: &Plaf, wit: &Witness) -> Result<(), io::Error> {
    let mut base_file = File::create(format!("{}.toml", name))?;
    let mut fixed_file = File::create(format!("{}_fixed.csv", name))?;
    let mut witness_file = File::create(format!("{}_witness.csv", name))?;

    write!(base_file, "{}", PlafDisplayBaseTOML(plaf))?;
    write!(fixed_file, "{}", PlafDisplayFixedCSV(plaf))?;
    write!(witness_file, "{}", WitnessDisplayCSV(wit))?;
    Ok(())
}

fn keccak_run(circuit_param: KeccakCircuit, k: u32){

    let super_circuit =
        keccak_super_circuit::<Fr>(circuit_param.bits.len(), circuit_param.output_len);
    let compiled = chiquitoSuperCircuit2Halo2(&super_circuit);

    let circuit = ChiquitoHalo2SuperCircuit::new(
        compiled,
        super_circuit.get_mapping().generate(circuit_param),
    );

    let prover = MockProver::<Fr>::run(k, &circuit, Vec::new()).unwrap();
    let result = prover.verify_par();

    println!("result = {:#?}", result);

    if let Err(failures) = &result {
        for failure in failures.iter() {
            println!("{}", failure);
        }
    }

}

fn keccak_plaf(circuit_param: KeccakCircuit, k: u32) {
    let super_circuit =
        keccak_super_circuit::<Fr>(circuit_param.bits.len(), circuit_param.output_len);
    let witness = super_circuit.get_mapping().generate(circuit_param);
    
    for wit_gen in witness.values(){
        let wit_gen = wit_gen.clone();

        let mut circuit = super_circuit.get_sub_circuits()[3].clone();
        circuit.columns.append(&mut super_circuit.get_sub_circuits()[0].columns);
        circuit.columns.append(&mut super_circuit.get_sub_circuits()[1].columns);
        circuit.columns.append(&mut super_circuit.get_sub_circuits()[2].columns);

        for (key, value) in super_circuit.get_sub_circuits()[0].fixed_assignments.iter(){
            circuit.fixed_assignments.insert(key.clone(), value.clone());
        }
        for (key, value) in super_circuit.get_sub_circuits()[1].fixed_assignments.iter(){
            circuit.fixed_assignments.insert(key.clone(), value.clone());
        }
        for (key, value) in super_circuit.get_sub_circuits()[2].fixed_assignments.iter(){
            circuit.fixed_assignments.insert(key.clone(), value.clone());
        }

        let (plaf, plaf_wit_gen) = chiquito2Plaf(circuit, k, false);
        
        let mut plaf = plaf;
        plaf.set_challange_alias(0, "r_keccak".to_string());
        let wit = plaf_wit_gen.generate(Some(wit_gen));
        write_files("keccak", &plaf, &wit).unwrap();
        
        // write_files("keccak_witness", &wit).unwrap();

        // debug only: print witness
        // println!("{}", WitnessDisplayCSV(&wit));
        // let plaf_circuit = PlafH2Circuit { plaf, wit };
        // let prover_plaf = MockProver::<Fr>::run(8, &plaf_circuit, Vec::new()).unwrap();

        // let result_plaf = prover_plaf.verify_par();
    }
}

fn main() {
    let circuit_param = KeccakCircuit {
        bits: vec![0, 0, 0, 0, 0, 0, 0, 0],
        output_len: 256,
        // bits: vec![
        //     0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1,
        // 1,     1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,
        // 1, 0, 1, 0,     1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1,
        // 1, 0, 1, 1, 1, 1, 1,     0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1,
        // 1, 0, 0, 0, 0, 0, 1, 0, 1, 0,     0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,
        // 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1,     1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1,
        // 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0,     1, 1, 0, 1, 0, 1, 1, 0, 1, 1,
        // 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1,     1, 1, 1, 0, 0, 0, 0,
        // 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0,     1, 0, 1, 0,
        // 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1,     1,
        // 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1,
        //     1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1,
        // 1,     1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0,
        // 0, 0, 0, 1,     0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1,
        // 1, 0, 1, 0, 1, 1, 0,     1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0,
        // 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,     1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0,
        // 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1,     1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0,
        // 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1,     0, 1, 0, 1, 0, 1, 1, 0, 1, 1,
        // 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1,     1, 1, 1, 1, 0, 0, 0,
        // 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,     1, 0, 1, 1,
        // 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1,     0,
        // 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0,
        //     0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1,
        // 1,     0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1,
        // 0, 0, 0, 0,     0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0,
        // 1, 0, 1, 0, 1, 1, 0,     1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1,
        // 0, 1, 1, 1, 1, 1, 0, 0, 0, 0,     0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1,
        // 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1,     1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1,
        // 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0,     0, 0, 0, 1, 0, 1, 1, 0, 1, 0,
        // 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0,     1, 1, 0, 1, 1, 1, 1,
        // 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0,     0, 0, 0, 1,
        // 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1,     1,
        // 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0,
        //     0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1,
        // 0,     1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1,
        // 1, 1, 1, 0,     0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0,
        // 0, 0, 1, 0, 1, 1, 0,     1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0,
        // 1, 0, 1, 1, 0, 1, 1, 1, 1, 1,     0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1,
        // 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1,     0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0,
        // 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0,     0, 0, 0, 0, 1, 0, 1, 0, 1, 0,
        // 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1,     0, 1, 1, 0, 1, 1, 1,
        // 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1,     0, 0, 0, 0,
        // 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1,     0,
        // 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1,
        //     1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0,
        // 1,     0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0,
        // 1, 1, 1, 1,     1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0,
        // 0, 0, 0, 1, 0, 1, 0,     1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1,
        // 0, 1, 0, 1, 1, 0, 1, 1, 1, 1,     1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0,
        // 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1,     1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0,
        // 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1,     1, 1, 1, 0, 0, 0, 0, 0, 1, 0,
        // 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1,     0, 1, 0, 1, 0, 1, 1,
        // 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1,     1, 1, 1, 0,
        // 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0,     1,
        // 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1,
        //     1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1,
        // 0,     1, 0, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0,
        // 1, 1, 0, 1,     1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1,
        // 1, 1, 0, 0, 0, 0, 0,     1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0,
        // 0, 1, 0, 1, 0, 1, 0, 1, 1, 0,     1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0,
        // 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,     1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1,
        // 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1,     1, 1, 1, 1, 0, 0, 0, 0, 0, 1,
        // 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1,     0, 1, 1, 0, 1, 0, 1,
        // 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0,     1, 1, 1, 1,
        // 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0,     0,
        // 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1,
        //     1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0,
        // 0,     0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0,
        // 1, 0, 1, 1,     0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1,
        // 1, 1, 1, 0, 0, 0, 0,     0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0,
        // 0, 1, 0, 1, 1, 0, 1, 0, 1, 1,     0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0,
        // 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0,     0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1,
        // 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0,     1, 1, 0, 1, 1, 1, 1, 1, 0, 0,
        // 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0,     0, 0, 0, 0, 1, 0, 1,
        // 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0,     1, 1, 0, 1,
        // 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0,     0,
        // 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1,
        //     1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0,
        // 0,     0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0,
        // 1, 1, 0, 1,     0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1,
        // 1, 0, 1, 1, 1, 1, 1,     0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1,
        // 1, 0, 0, 0, 0, 0, 1, 0, 1, ],
        // output_len: 20,
    };

    // keccak_run(circuit_param, 11);
    keccak_plaf(circuit_param, 11);


}
