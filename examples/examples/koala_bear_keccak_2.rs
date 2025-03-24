
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
use p3_examples::proofs::{prove_monty31_keccak, prove_monty31_poseidon2, report_result};
use p3_field::extension::BinomialExtensionField;
use p3_keccak_air::KeccakAir;
use p3_poseidon2_air::{RoundConstants, VectorizedPoseidon2Air};

const TRACE_LENGTH: u8 = 15;

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
    let num_hashes = trace_height / 24;
    type EF = BinomialExtensionField<KoalaBear, 4>;

    let proof_goal = {
        ProofObjective::Keccak(KeccakAir {})
    };

    let dft = DftChoice::Parallel(Radix2DitParallel::default());

    let result = prove_monty31_keccak::<_, EF, _, _>(proof_goal, dft, num_hashes);
    report_result(result);

}
