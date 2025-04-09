use std::marker::PhantomData;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use p3_challenger::{HashChallenger, SerializingChallenger32};
use p3_circle::CirclePcs;
use p3_commit::{ExtensionMmcs, Pcs};
use p3_field::extension::BinomialExtensionField;
use p3_fri::FriConfig;
use p3_keccak::Keccak256Hash;
use p3_matrix::dense::RowMajorMatrix;
use p3_merkle_tree::MerkleTreeMmcs;
use p3_mersenne_31::Mersenne31;
use p3_symmetric::{CompressionFunctionFromHasher, SerializingHasher32};
use plonky3_pcs::utilities::{LOG_TRACE_COLUMNS, LOG_TRACE_ROWS};
use rand::prelude::SmallRng;
use rand::{Rng, SeedableRng};

/// Main benchmark function setting up cryptographic configurations and executing benchmarks
fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = SmallRng::seed_from_u64(1);

    type Val = Mersenne31;
    type Challenge = BinomialExtensionField<Mersenne31, 3>;

    type ByteHash = Keccak256Hash;
    type FieldHash = SerializingHasher32<ByteHash>;
    let byte_hash = ByteHash {};
    let field_hash = FieldHash::new(byte_hash);

    type MyCompress = CompressionFunctionFromHasher<ByteHash, 2, 32>;
    let compress = MyCompress::new(byte_hash);

    type ValMmcs = MerkleTreeMmcs<Val, u8, FieldHash, MyCompress, 32>;
    let val_mmcs = ValMmcs::new(field_hash, compress);

    type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;
    let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());

    type Challenger = SerializingChallenger32<Val, HashChallenger<u8, ByteHash, 32>>;

    let trace = RowMajorMatrix::rand(&mut rng, 1 << LOG_TRACE_ROWS, 1 << LOG_TRACE_COLUMNS);

    let fri_test_cases = [
        (1, 256), // log_blowup: 1, num_queries: 256
                  //  (3, 64),  // log_blowup: 3, num_queries: 64
    ];

    for &(log_blowup, num_queries) in &fri_test_cases {
        let fri_config = FriConfig {
            log_blowup,
            log_final_poly_len: 0,
            num_queries,
            proof_of_work_bits: 16,
            mmcs: challenge_mmcs.clone(),
        };

        type Pcs = CirclePcs<Val, ValMmcs, ChallengeMmcs>;
        let pcs = Pcs {
            mmcs: val_mmcs.clone(),
            fri_config,
            _phantom: PhantomData,
        };

        let trace_domain =
            <Pcs as p3_commit::Pcs<Challenge, Challenger>>::natural_domain_for_degree(
                &pcs,
                1 << LOG_TRACE_ROWS,
            );

        let mut group = c.benchmark_group("Circle PCS Benchmarks");
        group.sample_size(10); // Limit benchmark to 10 samples

        group.bench_with_input(
            BenchmarkId::new(
                "Commit",
                format!("log_blowup: {}, num_queries: {}", log_blowup, num_queries),
            ),
            &(),
            |b, _| {
                b.iter(|| {
                    <Pcs as p3_commit::Pcs<Challenge, Challenger>>::commit(
                        &pcs,
                        vec![(trace_domain, trace.clone())],
                    );
                })
            },
        );
        let (comm, data) = <Pcs as p3_commit::Pcs<Challenge, Challenger>>::commit(
            &pcs,
            vec![(trace_domain, trace.clone())],
        );

        let zeta: Challenge = rng.random();

        let mut chal = Challenger::from_hasher(vec![], byte_hash);
        let (opened_values, opening_proof) = pcs.open(vec![(&data, vec![vec![zeta]])], &mut chal);

        let config = bincode::config::standard()
            .with_little_endian()
            .with_fixed_int_encoding();

        let comm_bytes =
            bincode::serde::encode_to_vec(comm.clone(), config).expect("Failed to serialize comm");
        let opened_values_bytes = bincode::serde::encode_to_vec(opened_values.clone(), config)
            .expect("Failed to serialize opened_values");
        let opening_proof_bytes = bincode::serde::encode_to_vec(opening_proof.clone(), config)
            .expect("Failed to serialize opening_proof");

        println!(
            "Proof size: {} bytes",
            opening_proof_bytes.len() + comm_bytes.len() + opened_values_bytes.len()
        );

        group.bench_with_input(
            BenchmarkId::new(
                "Open",
                format!("log_blowup: {}, num_queries: {}", log_blowup, num_queries),
            ),
            &(),
            |b, _| {
                b.iter(|| {
                    pcs.open(vec![(&data, vec![vec![zeta]])], &mut chal);
                })
            },
        );
        let mut chal = Challenger::from_hasher(vec![], byte_hash);

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
                            comm,
                            vec![(trace_domain, vec![(zeta, opened_values[0][0][0].clone())])],
                        )],
                        &opening_proof,
                        &mut chal,
                    );
                })
            },
        );
    }
}

criterion_group!(benches, criterion_benchmark);

#[cfg(all(
    feature = "nightly-features",
    target_arch = "x86_64",
    target_feature = "avx512f"
))]
criterion_main!(benches);
