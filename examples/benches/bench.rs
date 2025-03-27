use criterion::{black_box, criterion_group, criterion_main, Criterion};
use p3_challenger::{CanSample, DuplexChallenger};
use p3_commit::{ExtensionMmcs, Pcs, PolynomialSpace};
use p3_dft::Radix2DitParallel;
use p3_examples::dfts::DftChoice;
use p3_examples::proofs::get_poseidon2_mmcs;
use p3_field::extension::BinomialExtensionField;
use p3_fri::{FriConfig, TwoAdicFriPcs};
use p3_koala_bear::{KoalaBear, Poseidon2KoalaBear};
use p3_matrix::Matrix;
use p3_matrix::dense::RowMajorMatrix;
use p3_uni_stark::{StarkConfig, StarkGenericConfig, Val};
use rand::{SeedableRng, rng};
use rand_chacha::ChaCha8Rng;
use std::time::Instant;

const LOG_TRACE_ROWS: usize = 10;
const LOG_TRACE_COLUMNS: usize = 11;

/// Commit to trace data benchmark
fn commit_pcs<SC>(config: &SC, challenger: &mut SC::Challenger, trace: RowMajorMatrix<Val<SC>>)
    where
        SC: StarkGenericConfig,
{
    let degree = trace.height();

    let pcs = config.pcs();
    let trace_domain = pcs.natural_domain_for_degree(degree);

    // Benchmark commit to trace data
    let (_trace_commit, _trace_data) = pcs.commit(vec![(trace_domain, trace.clone())]);
}
/*
/// Open commitments benchmark
fn open_pcs<SC>(config: &SC, challenger: &mut SC::Challenger, trace_data: RowMajorMatrix<Val<SC>>) -> u128
    where
        SC: StarkGenericConfig,
{
    let degree = trace_data.height();

    let pcs = config.pcs();
    let trace_domain = pcs.natural_domain_for_degree(degree);

    // Sample challenge
    let zeta: SC::Challenge = challenger.sample();
    let zeta_next = trace_domain.next_point(zeta).unwrap();

    // Benchmark open commitments
    let open_start = Instant::now();
    let (_opened_values, _opening_proof) = pcs.open(vec![(&trace_data, vec![vec![zeta, zeta_next]])], challenger);
    open_start.elapsed().as_micros() // Return open duration in microseconds
}


 */
fn bench_commit(c: &mut Criterion) {
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

    let pcs = TwoAdicFriPcs::new(dft, val_mmcs, fri_config);

    // Constructs the STARK proof system and calls commit_pcs.
    let config = StarkConfig::new(pcs);
    let mut challenger: DuplexChallenger<Val, Poseidon2KoalaBear<24>, 24, 16> =
        DuplexChallenger::new(perm24.clone());

    let mut group = c.benchmark_group("PCS Commit");
    group.sample_size(10); // Limit to 10 samples

    // Perform commit benchmarking
    group.bench_function("commit", |b| {
        b.iter(|| {
            commit_pcs(&config, &mut challenger, trace.clone());
        })
    });
}
/*
fn bench_open(c: &mut Criterion) {
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

    let pcs = TwoAdicFriPcs::new(dft, val_mmcs, fri_config);

    // Constructs the STARK proof system and calls open_pcs.
    let config = StarkConfig::new(pcs);
    let mut challenger: DuplexChallenger<Val, Poseidon2KoalaBear<24>, 24, 16> =
        DuplexChallenger::new(perm24.clone());

    // Commit to trace data first (needed for open)
    let (_trace_commit, trace_data) = pcs.commit(vec![(pcs.natural_domain_for_degree(trace.height()), trace.clone())]);

    // Perform open benchmarking
    c.bench_function("open", |b| {
        b.iter(|| {
            let open_duration = open_pcs(&config, &mut challenger, trace_data.clone());
            black_box(open_duration); // Prevent optimization
        })
    });
}

 */

criterion_group!(benches, bench_commit);
criterion_main!(benches);
