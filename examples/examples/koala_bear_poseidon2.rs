
use p3_koala_bear::{GenericPoseidon2LinearLayersKoalaBear, KoalaBear, Poseidon2KoalaBear};
use rand::{rng};
use tracing_forest::ForestLayer;
use tracing_forest::util::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};
use p3_dft::Radix2DitParallel;
use p3_examples::airs::ProofObjective;
use p3_examples::dfts::DftChoice;
use p3_examples::proofs::{prove_monty31_poseidon2, report_result};
use p3_field::extension::BinomialExtensionField;
use p3_poseidon2_air::{RoundConstants, VectorizedPoseidon2Air};

const TRACE_LENGTH: u8 = 16;

// General constants for constructing the Poseidon2 AIR.
const P2_WIDTH: usize = 16;
const P2_HALF_FULL_ROUNDS: usize = 4;
const P2_LOG_VECTOR_LEN: u8 = 3;
const P2_VECTOR_LEN: usize = 1 << P2_LOG_VECTOR_LEN;

fn main() {

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    Registry::default()
        .with(env_filter)
        .with(ForestLayer::default())
        .init();

    type Val = KoalaBear;

    let trace_height = 1 << TRACE_LENGTH;
    let num_hashes = trace_height << P2_LOG_VECTOR_LEN;

    let proof_goal = {
        let constants = RoundConstants::from_rng(&mut rng());

        // Field specific constants for constructing the Poseidon2 AIR.
        const SBOX_DEGREE: u64 = 3;
        const SBOX_REGISTERS: usize = 0;
        const PARTIAL_ROUNDS: usize = 20;

        let p2_air: VectorizedPoseidon2Air<
            KoalaBear,
            GenericPoseidon2LinearLayersKoalaBear,
            P2_WIDTH,
            SBOX_DEGREE,
            SBOX_REGISTERS,
            P2_HALF_FULL_ROUNDS,
            PARTIAL_ROUNDS,
            P2_VECTOR_LEN,
        > = VectorizedPoseidon2Air::new(constants);
        ProofObjective::Poseidon2(p2_air)
    };

    let dft = DftChoice::Parallel(Radix2DitParallel::default());

    let perm16 = Poseidon2KoalaBear::<16>::new_from_rng_128(&mut rng());
    let perm24 = Poseidon2KoalaBear::<24>::new_from_rng_128(&mut rng());
    let result = prove_monty31_poseidon2::<_, Val, _, _, _, _>(
        proof_goal, dft, num_hashes, perm16, perm24,
    );
    report_result(result);

}
