use criterion::{Criterion, criterion_group, criterion_main};
use p3_challenger::{CanSample, DuplexChallenger};
use p3_commit::{ExtensionMmcs, Pcs, PolynomialSpace};
use p3_dft::Radix2DitParallel;
use p3_examples::dfts::DftChoice;
use p3_examples::proofs::{get_poseidon2_mmcs};
use p3_field::extension::BinomialExtensionField;
use p3_fri::{FriConfig, TwoAdicFriPcs};
use p3_koala_bear::{KoalaBear, Poseidon2KoalaBear};
use p3_matrix::Matrix;
use p3_matrix::dense::RowMajorMatrix;
use p3_uni_stark::{StarkConfig, StarkGenericConfig, Val};
use rand::{SeedableRng, rng};
use rand_chacha::ChaCha8Rng;
use p3_examples::utilities::{Proof, prove_without_air, report_proof_size_example};

/// Constants defining the size of the trace matrix
const LOG_TRACE_ROWS: usize = 19;
const LOG_TRACE_COLUMNS: usize = 11;

/// Benchmark function for committing and opening a trace matrix using Polynomial Commitment Scheme (PCS)
fn bench_commit_open<SC>(
    config: &SC,
    challenger: &mut SC::Challenger,
    trace: RowMajorMatrix<Val<SC>>,
    c: &mut Criterion,
) where
    SC: StarkGenericConfig,
{
    let pcs = config.pcs();
    let trace_domain = pcs.natural_domain_for_degree(trace.height());

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

    let zeta: SC::Challenge = challenger.sample();
    let zeta_next = trace_domain.next_point(zeta).unwrap();

    // Benchmark the opening step
    group.bench_function("Open", |b| {
        b.iter(|| {
            pcs.open(vec![(&trace_data, vec![vec![zeta, zeta_next]])], challenger);
        })
    });

}


/// Benchmark function for Verification
fn bench_verify<SC>(
    config: &SC,
    challenger: &mut SC::Challenger,
    proof: &Proof<SC>,
    c: &mut Criterion,
) where
    SC: StarkGenericConfig,
{

    let pcs = config.pcs();
    let trace_domain = pcs.natural_domain_for_degree(1 << proof.degree_bits);

    let zeta: SC::Challenge = challenger.sample();
    let zeta_next = trace_domain.next_point(zeta).unwrap();

    let mut group = c.benchmark_group("PCS Benchmarks");
    group.sample_size(10); // Limit benchmark to 10 samples

    group.bench_function("Verify", |b| {
        b.iter(|| {
            let _=pcs.verify(
                vec![(
                    proof.commitments.trace.clone(),
                    vec![(
                        trace_domain,
                        vec![
                            (zeta, proof.opened_values.trace_local.clone()),
                            (zeta_next, proof.opened_values.trace_next.clone()),
                        ],
                    )],
                )],
                &proof.opening_proof,
                challenger,
            );
        })
    });
}

/// Main benchmark function setting up cryptographic configurations and executing benchmarks
fn criterion_benchmark(c: &mut Criterion) {
    // Defines the field type and cryptographic settings
    type EF = BinomialExtensionField<KoalaBear, 4>;

    let dft = DftChoice::Parallel(Radix2DitParallel::default());

    let perm16 = Poseidon2KoalaBear::<16>::new_from_rng_128(&mut rng());
    let perm24 = Poseidon2KoalaBear::<24>::new_from_rng_128(&mut rng());
    // Generate Merkle commitment schemes
    let val_mmcs = get_poseidon2_mmcs::<KoalaBear, _, _>(perm16, perm24.clone());

    let challenge_mmcs = ExtensionMmcs::<KoalaBear, EF, _>::new(val_mmcs.clone());
    let fri_config = FriConfig {
        log_blowup: 3,
        log_final_poly_len: 0,
        num_queries: 64,
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

    let config = StarkConfig::new(pcs);

    let mut proof_challenger: DuplexChallenger<KoalaBear, Poseidon2KoalaBear<24>, 24, 16> =
        DuplexChallenger::new(perm24.clone());
    let mut verif_challenger: DuplexChallenger<KoalaBear, Poseidon2KoalaBear<24>, 24, 16> = DuplexChallenger::new(perm24);


    bench_commit_open(&config, &mut proof_challenger, trace.clone(), c);

    let proof = prove_without_air(&config, &mut proof_challenger, trace.clone());
    report_proof_size_example(&proof);

    bench_verify(&config, &mut verif_challenger, &proof,c)

}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
