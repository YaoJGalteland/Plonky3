use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use p3_challenger::{CanSample, DuplexChallenger};
use p3_commit::{ExtensionMmcs, Pcs, PolynomialSpace};
use p3_dft::Radix2DitParallel;
use p3_field::Field;
use p3_field::extension::BinomialExtensionField;
use p3_fri::{FriConfig, TwoAdicFriPcs};
use p3_koala_bear::{KoalaBear, Poseidon2ExternalLayerKoalaBear, Poseidon2InternalLayerKoalaBear};
use p3_matrix::Matrix;
use p3_matrix::dense::RowMajorMatrix;
use p3_merkle_tree::MerkleTreeMmcs;
use p3_poseidon2::Poseidon2;
use p3_symmetric::{PaddingFreeSponge, TruncatedPermutation};
use p3_uni_stark::{StarkConfig, StarkGenericConfig, Val};
use plonky3_pcs::utilities::{Proof, prove_random_trace, report_proof_size_example};
use rand::SeedableRng;
use rand::prelude::SmallRng;

/// Constants defining the size of the trace matrix
const LOG_TRACE_ROWS: usize = 19;
const LOG_TRACE_COLUMNS: usize = 11;

type F = KoalaBear;
type Challenge = BinomialExtensionField<F, 4>;
type Dft = Radix2DitParallel<F>;
type Perm16 =
    Poseidon2<F, Poseidon2ExternalLayerKoalaBear<16>, Poseidon2InternalLayerKoalaBear<16>, 16, 3>;
type Perm24 =
    Poseidon2<F, Poseidon2ExternalLayerKoalaBear<24>, Poseidon2InternalLayerKoalaBear<24>, 24, 3>;
type MyHash = PaddingFreeSponge<Perm24, 24, 16, 8>;
type MyCompress = TruncatedPermutation<Perm16, 2, 8, 16>;
type ValMmcs = MerkleTreeMmcs<<F as Field>::Packing, <F as Field>::Packing, MyHash, MyCompress, 8>;
type ChallengeMmcs = ExtensionMmcs<F, Challenge, ValMmcs>;
type Challenger = DuplexChallenger<F, Perm24, 24, 16>;
type PCS = TwoAdicFriPcs<F, Dft, ValMmcs, ChallengeMmcs>;
type MyConfig = StarkConfig<PCS, Challenge, Challenger>;

/// Benchmark function for committing and opening a random trace matrix
fn bench_commit_open<SC>(
    config: &SC,
    challenger: &mut SC::Challenger,
    trace: RowMajorMatrix<Val<SC>>,
    log_blowup: usize,
    num_queries: usize,
    c: &mut Criterion,
) where
    SC: StarkGenericConfig,
{
    let pcs = config.pcs();
    let trace_domain = pcs.natural_domain_for_degree(trace.height());

    let mut group = c.benchmark_group("PCS Benchmarks");
    group.sample_size(10); // Limit benchmark to 10 samples

    // Benchmark the commitment step
    group.bench_with_input(
        BenchmarkId::new(
            "Commit",
            format!("log_blowup: {}, num_queries: {}", log_blowup, num_queries),
        ),
        &(),
        |b, _| {
            b.iter(|| {
                pcs.commit(vec![(trace_domain, trace.clone())]);
            })
        },
    );
    // Commit to the trace matrix and store the commitment data
    let (_trace_commit, trace_data) = pcs.commit(vec![(trace_domain, trace.clone())]);

    let zeta: SC::Challenge = challenger.sample();
    let zeta_next = trace_domain.next_point(zeta).unwrap();

    // Benchmark the opening step
    group.bench_with_input(
        BenchmarkId::new(
            "Open",
            format!("log_blowup: {}, num_queries: {}", log_blowup, num_queries),
        ),
        &(),
        |b, _| {
            b.iter(|| {
                pcs.open(vec![(&trace_data, vec![vec![zeta, zeta_next]])], challenger);
            })
        },
    );
}

/// This function benchmarks the verification step for the provided proof.
fn bench_verify<SC>(
    config: &SC,
    challenger: &mut SC::Challenger,
    proof: &Proof<SC>,
    log_blowup: usize,
    num_queries: usize,
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

    // Benchmark the verify step
    group.bench_with_input(
        BenchmarkId::new(
            "Verify",
            format!("log_blowup: {}, num_queries: {}", log_blowup, num_queries),
        ),
        &(),
        |b, _| {
            b.iter(|| {
                let _ = pcs.verify(
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
        },
    );
}

/// Main benchmark function setting up cryptographic configurations and executing benchmarks
fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = SmallRng::seed_from_u64(1);

    let dft = Dft::default();

    let perm16 = Perm16::new_from_rng_128(&mut rng);
    let perm24 = Perm24::new_from_rng_128(&mut rng);

    let hash = MyHash::new(perm24.clone());
    let compress = MyCompress::new(perm16.clone());

    let val_mmcs = ValMmcs::new(hash, compress);

    let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());

    // Generate a random trace matrix
    let trace = RowMajorMatrix::rand(&mut rng, 1 << LOG_TRACE_ROWS, 1 << LOG_TRACE_COLUMNS);
    println!(
        "trace dimension: rows={:?}, columns={:?}",
        trace.values.len() / trace.width,
        trace.width
    );

    let fri_test_cases = [
        (1, 256), // log_blowup: 1, num_queries: 256
        (3, 64),  // log_blowup: 3, num_queries: 64
    ];

    for &(log_blowup, num_queries) in &fri_test_cases {
        let fri_config = FriConfig {
            log_blowup,
            log_final_poly_len: 0,
            num_queries,
            proof_of_work_bits: 1,
            mmcs: challenge_mmcs.clone(),
        };

        let pcs = TwoAdicFriPcs::new(dft.clone(), val_mmcs.clone(), fri_config);

        let config = MyConfig::new(pcs);

        let mut proof_challenger = Challenger::new(perm24.clone());
        let mut verif_challenger = Challenger::new(perm24.clone());

        // Run the commit/open benchmark
        bench_commit_open(
            &config,
            &mut proof_challenger,
            trace.clone(),
            log_blowup,
            num_queries,
            c,
        );

        let proof = prove_random_trace(&config, &mut proof_challenger, trace.clone());
        report_proof_size_example(&proof);

        // Run the verification benchmark
        bench_verify(
            &config,
            &mut verif_challenger,
            &proof,
            log_blowup,
            num_queries,
            c,
        )
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
