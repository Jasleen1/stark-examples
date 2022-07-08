use std::time::Instant;
use winterfell::{
    math::{fields::f128::BaseElement as Felt, FieldElement},
    Air, AirContext, Assertion, ByteWriter, EvaluationFrame, FieldExtension, HashFunction,
    ProofOptions, Prover, Serializable, StarkProof, Trace, TraceInfo, TraceTable,
    TransitionConstraintDegree,
};

// CONSTANTS
// ================================================================================================

const TRACE_WIDTH: usize = 2;

// MAIN FUNCTION
// ================================================================================================

pub fn main() {
    let n = 128;

    // compute result
    let now = Instant::now();
    let result = compute_fib_term(n);
    println!("Computed result in {} ms", now.elapsed().as_millis());

    // specify parameters for the STARK protocol
    let stark_params = ProofOptions::new(
        40,
        4,
        21,
        HashFunction::Blake3_256,
        FieldExtension::None,
        8,
        64,
    );

    // instantiate the prover
    let prover = FibProver::new(stark_params);

    // build execution trace
    let now = Instant::now();
    let trace = prover.build_trace(n);
    println!("Built execution trace in {} ms", now.elapsed().as_millis());
    assert_eq!(result, trace.get(1, n / 2 - 1));

    // generate the proof
    let now = Instant::now();
    let proof = prover.prove(trace).unwrap();
    println!("Generated proof in {} ms", now.elapsed().as_millis());

    // serialize proof and check security level
    let proof_bytes = proof.to_bytes();
    println!("Proof size: {:.1} KB", proof_bytes.len() as f64 / 1024f64);
    println!("Proof security: {} bits", proof.security_level(true));

    // deserialize proof
    let parsed_proof = StarkProof::from_bytes(&proof_bytes).unwrap();
    assert_eq!(proof, parsed_proof);

    // initialize public inputs
    let pub_inputs = compute_fib_term(n);

    // verify the proof
    let now = Instant::now();
    match winterfell::verify::<FibAir>(proof, pub_inputs) {
        Ok(_) => println!(
            "Proof verified in {:.1} ms",
            now.elapsed().as_micros() as f64 / 1000f64
        ),
        Err(msg) => println!("Something went wrong! {}", msg),
    }
}

// Fibonacci AIR
// ================================================================================================

pub struct FibAir {
    context: AirContext<Felt>,
    result: Felt,
}

impl Air for FibAir {
    type BaseField = Felt;
    type PublicInputs = Felt;

    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, pub_inputs: Self::BaseField, options: ProofOptions) -> Self {
        let degrees = vec![
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(1),
        ];
        assert_eq!(TRACE_WIDTH, trace_info.width());
        FibAir {
            context: AirContext::new(trace_info, degrees, options),
            result: pub_inputs,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    fn evaluate_transition<E: FieldElement<BaseField = Self::BaseField> + From<Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        _periodic_values: &[E],
        result: &mut [E],
    ) {
        let current = frame.current();
        let next = frame.next();
        // expected state width is 2 field elements
        debug_assert_eq!(TRACE_WIDTH, current.len());
        debug_assert_eq!(TRACE_WIDTH, next.len());

        // constraints of Fibonacci sequence (2 terms per step):
        // s_{0, i+1} = s_{0, i} + s_{1, i}
        // s_{1, i+1} = s_{1, i} + s_{0, i+1}
        result[0] = next[0] - (current[0] + current[1]);
        result[1] = next[1] - (current[1] + next[0]);
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        // a valid Fibonacci sequence should start with two ones and terminate with
        // the expected result
        let last_step = self.trace_length() - 1;
        vec![
            Assertion::single(0, 0, Self::BaseField::ONE),
            Assertion::single(1, 0, Self::BaseField::ONE),
            Assertion::single(1, last_step, self.result),
        ]
    }
}

// PROVER
// ================================================================================================

// FIBONACCI PROVER
// ================================================================================================

pub struct FibProver {
    options: ProofOptions,
}

impl FibProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }

    /// Builds an execution trace for computing a Fibonacci sequence of the specified length such
    /// that each row advances the sequence by 2 terms.
    pub fn build_trace(&self, sequence_length: usize) -> TraceTable<Felt> {
        assert!(
            sequence_length.is_power_of_two(),
            "sequence length must be a power of 2"
        );

        let mut trace = TraceTable::new(TRACE_WIDTH, sequence_length / 2);
        trace.fill(
            |state| {
                state[0] = Felt::ONE;
                state[1] = Felt::ONE;
            },
            |_, state| {
                state[0] += state[1];
                state[1] += state[0];
            },
        );

        trace
    }
}

impl Prover for FibProver {
    type BaseField = Felt;
    type Air = FibAir;
    type Trace = TraceTable<Felt>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> Felt {
        let last_step = trace.length() - 1;
        trace.get(1, last_step)
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}

/// HELPERS

/// Computes the nth term of the fibonacci sequence.
pub fn compute_fib_term(n: usize) -> Felt {
    let mut t0 = Felt::ONE;
    let mut t1 = Felt::ONE;

    for _ in 0..(n - 1) {
        t1 = t0 + t1;
        core::mem::swap(&mut t0, &mut t1);
    }

    t1
}
