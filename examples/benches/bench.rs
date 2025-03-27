use criterion::{criterion_group, criterion_main, Criterion};
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

/// Constants defining the size of the trace matrix
const LOG_TRACE_ROWS: usize = 19;
const LOG_TRACE_COLUMNS: usize = 11;

/// Benchmark function for committing and opening a trace matrix using Polynomial Commitment Scheme (PCS)
fn commit_open<SC>(config: &SC, challenger: &mut SC::Challenger, trace: RowMajorMatrix<Val<SC>>, c: &mut Criterion)
    where
        SC: StarkGenericConfig,
{
    let degree = trace.height();
    let pcs = config.pcs();
    let trace_domain = pcs.natural_domain_for_degree(degree);

    let mut group = c.benchmark_group("PCS Benchmarks");
    group.sample_size(10); // Limit benchmark to 10 samples

    // Benchmark the commitment step
    group.bench_function("Commit", |b| {
        b.iter(|| {
            pcs.commit(vec![(trace_domain, trace.clone())]);
        })
    });

    // Commit to the trace matrix and store the commitment data
    let (_trace_commit, trace_data) = pcs.commit(vec![(trace_domain, trace.clone())]);

    // Sample a challenge value
    let zeta: SC::Challenge = challenger.sample();
    let zeta_next = trace_domain.next_point(zeta).unwrap();

    // Benchmark the opening step
    group.bench_function("Open", |b| {
        b.iter(|| {
            pcs.open(vec![(&trace_data, vec![vec![zeta, zeta_next]])], challenger);
        })
    });
}

/// Main benchmark function setting up cryptographic configurations and executing benchmarks
fn criterion_benchmark(c: &mut Criterion) {
    // Defines the field type and cryptographic settings
    type Val = KoalaBear;
    type Challenge = BinomialExtensionField<Val, 4>;

    // Configures the discrete Fourier transform (DFT) and cryptographic permutations
    let dft = DftChoice::Parallel(Radix2DitParallel::default());
    let perm16 = Poseidon2KoalaBear::<16>::new_from_rng_128(&mut rng());
    let perm24 = Poseidon2KoalaBear::<24>::new_from_rng_128(&mut rng());

    // Generate Merkle commitment schemes
    let val_mmcs = get_poseidon2_mmcs::<Val, _, _>(perm16.clone(), perm24.clone());
    let challenge_mmcs = ExtensionMmcs::<Val, Challenge, _>::new(val_mmcs.clone());

    // Create a FRI configuration with InvRate = 2 (log_blowup = 1)
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

    // Set up the Polynomial Commitment Scheme (PCS)
    let pcs = TwoAdicFriPcs::new(dft, val_mmcs, fri_config);

    // Initialize the STARK proof system configuration
    let config = StarkConfig::new(pcs);
    let mut challenger: DuplexChallenger<Val, Poseidon2KoalaBear<24>, 24, 16> =
        DuplexChallenger::new(perm24.clone());

    // Execute the benchmark
    commit_open(&config, &mut challenger, trace.clone(), c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
