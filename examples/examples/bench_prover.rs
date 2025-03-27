use clap::Parser;
use p3_baby_bear::{BabyBear, GenericPoseidon2LinearLayersBabyBear, Poseidon2BabyBear};
use p3_blake3_air::Blake3Air;
use p3_challenger::{CanSample, DuplexChallenger};
use p3_commit::{ExtensionMmcs, Pcs, PolynomialSpace};
use p3_dft::Radix2DitParallel;
use p3_examples::airs::ProofObjective;
use p3_examples::dfts::DftChoice;
use p3_examples::parsers::{DftOptions, FieldOptions, MerkleHashOptions, ProofOptions};
use p3_examples::proofs::{
    get_poseidon2_mmcs, prove_m31_keccak, prove_m31_poseidon2, prove_monty31_keccak,
    prove_monty31_poseidon2, report_result,
};
use p3_field::extension::BinomialExtensionField;
use p3_fri::{FriConfig, TwoAdicFriPcs, create_benchmark_fri_config};
use p3_keccak_air::KeccakAir;
use p3_koala_bear::{GenericPoseidon2LinearLayersKoalaBear, KoalaBear, Poseidon2KoalaBear};
use p3_matrix::Matrix;
use p3_matrix::dense::RowMajorMatrix;
use p3_mersenne_31::{GenericPoseidon2LinearLayersMersenne31, Mersenne31, Poseidon2Mersenne31};
use p3_monty_31::dft::RecursiveDft;
use p3_poseidon2_air::{RoundConstants, VectorizedPoseidon2Air};
use p3_uni_stark::{StarkConfig, StarkGenericConfig, Val};
use rand::{SeedableRng, rng};
use rand_chacha::ChaCha8Rng;
use tracing::info_span;
use tracing_forest::ForestLayer;
use tracing_forest::util::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

// Constants for trace matrix size
const LOG_TRACE_ROWS: usize = 19;
const LOG_TRACE_COLUMNS: usize = 11;

/// Evaluate the pref commit and open of the Polynomial Commitment Scheme (PCS)
fn prove_pcs<SC>(config: &SC, challenger: &mut SC::Challenger, trace: RowMajorMatrix<Val<SC>>)
where
    SC: StarkGenericConfig,
{
    let degree = trace.height();

    let pcs = config.pcs();
    let trace_domain = pcs.natural_domain_for_degree(degree);

    // Commit to trace data
    let (_trace_commit, trace_data) = info_span!("commit to trace data")
        .in_scope(|| pcs.commit(vec![(trace_domain, trace.clone())]));

    // Sample challenge
    let zeta: SC::Challenge = challenger.sample();

    // Open commitments
    let (_opened_values, _opening_proof) =
        info_span!("open").in_scope(|| pcs.open(vec![(&trace_data, vec![vec![zeta]])], challenger));
}
fn main() {
    // Initializes logging
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    Registry::default()
        .with(env_filter)
        .with(ForestLayer::default())
        .init();

    // Defines the field type and cryptographic settings.
    type Val = KoalaBear;
    type Challenge = BinomialExtensionField<Val, 4>;

    // Configures the discrete Fourier transform and cryptographic permutations.
    let dft = DftChoice::Parallel(Radix2DitParallel::default());
    let perm16 = Poseidon2KoalaBear::<16>::new_from_rng_128(&mut rng());
    let perm24 = Poseidon2KoalaBear::<24>::new_from_rng_128(&mut rng());

    // Generate Merkle commitment schemes
    let val_mmcs = get_poseidon2_mmcs::<Val, _, _>(perm16.clone(), perm24.clone());
    let challenge_mmcs = ExtensionMmcs::<Val, Challenge, _>::new(val_mmcs.clone());

    // Create a FRI config for InvRate = 2, that is, log_blowup = 1
    let fri_config = FriConfig {
        log_blowup: 1,
        log_final_poly_len: 0,
        num_queries: 256,
        proof_of_work_bits: 1,
        mmcs: challenge_mmcs,
    };

    // Generate a random trace matrix
    let mut rng = ChaCha8Rng::from_seed([0; 32]);
    let trace = RowMajorMatrix::rand(&mut rng, 1 << LOG_TRACE_ROWS, 1 << LOG_TRACE_COLUMNS);
    println!(
        "trace dimension: rows={:?}, columns={:?}",
        trace.values.len() / trace.width,
        trace.width
    );

    let pcs = TwoAdicFriPcs::new(dft, val_mmcs, fri_config);

    // Constructs the STARK proof system and calls prove_pcs.
    let config = StarkConfig::new(pcs);
    let mut proof_challenger: DuplexChallenger<Val, Poseidon2KoalaBear<24>, 24, 16> =
        DuplexChallenger::new(perm24.clone());

    let _proof = prove_pcs(&config, &mut proof_challenger, trace.clone());
}
