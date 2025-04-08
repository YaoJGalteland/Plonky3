#[cfg(test)]
mod tests {
    use alloc::vec;
    use core::marker::PhantomData;

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
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};
    use tracing::info_span;
    use tracing::level_filters::LevelFilter;
    use tracing_forest::ForestLayer;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::{EnvFilter, Registry};

    use crate::utilities::{LOG_TRACE_COLUMNS, LOG_TRACE_ROWS};

    fn circle_pcs(log_blowup: usize, num_queries: usize) {
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

        let env_filter = EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy();

        Registry::default()
            .with(env_filter)
            .with(ForestLayer::default())
            .init();

        let fri_config = FriConfig {
            log_blowup,
            log_final_poly_len: 0,
            num_queries,
            proof_of_work_bits: 0,
            mmcs: challenge_mmcs,
        };

        type Pcs = CirclePcs<Val, ValMmcs, ChallengeMmcs>;
        let pcs = Pcs {
            mmcs: val_mmcs,
            fri_config,
            _phantom: PhantomData,
        };

        let trace_domain =
            <Pcs as p3_commit::Pcs<Challenge, Challenger>>::natural_domain_for_degree(
                &pcs,
                1 << LOG_TRACE_ROWS,
            );

        let trace = RowMajorMatrix::rand(&mut rng, 1 << LOG_TRACE_ROWS, 1 << LOG_TRACE_COLUMNS);

        let (comm, data) = info_span!("commit to trace data").in_scope(|| {
            <Pcs as p3_commit::Pcs<Challenge, Challenger>>::commit(
                &pcs,
                vec![(trace_domain, trace)],
            )
        });

        let zeta: Challenge = rng.random();

        let mut chal = Challenger::from_hasher(vec![], byte_hash);
        let (values, proof) =
            info_span!("open").in_scope(|| pcs.open(vec![(&data, vec![vec![zeta]])], &mut chal));

        let mut chal = Challenger::from_hasher(vec![], byte_hash);

        info_span!("Verify").in_scope(|| {
            pcs.verify(
                vec![(
                    comm,
                    vec![(trace_domain, vec![(zeta, values[0][0][0].clone())])],
                )],
                &proof,
                &mut chal,
            )
            .expect("verify err");

            tracing::info!("Proof Verified Successfully");
        });
    }

    #[test]
    fn test_circle_pcs_inv_rate2() {
        circle_pcs(1, 256);
    }
}
