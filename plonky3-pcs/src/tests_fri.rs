#[cfg(test)]
mod tests {
    use p3_challenger::DuplexChallenger;
    use p3_commit::ExtensionMmcs;
    use p3_dft::Radix2DitParallel;
    use p3_field::Field;
    use p3_field::extension::BinomialExtensionField;
    use p3_fri::{FriConfig, TwoAdicFriPcs};
    use p3_koala_bear::{
        KoalaBear, Poseidon2ExternalLayerKoalaBear, Poseidon2InternalLayerKoalaBear,
    };
    use p3_matrix::dense::RowMajorMatrix;
    use p3_merkle_tree::MerkleTreeMmcs;
    use p3_poseidon2::Poseidon2;
    use p3_symmetric::{PaddingFreeSponge, TruncatedPermutation};
    use p3_uni_stark::StarkConfig;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use tracing::level_filters::LevelFilter;
    use tracing_forest::ForestLayer;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::{EnvFilter, Registry};

    use crate::utilities::{
        LOG_TRACE_COLUMNS, LOG_TRACE_ROWS, prove_random_trace, report_proof_size_example,
        verify_random_trace,
    };

    // Test PCS for different FRI config
    fn test_fri(log_blowup: usize, num_queries: usize) {
        type Val = KoalaBear;
        type Challenge = BinomialExtensionField<Val, 4>;
        type Dft = Radix2DitParallel<Val>;

        type Perm16 = Poseidon2<
            Val,
            Poseidon2ExternalLayerKoalaBear<16>,
            Poseidon2InternalLayerKoalaBear<16>,
            16,
            3,
        >;
        type Perm24 = Poseidon2<
            Val,
            Poseidon2ExternalLayerKoalaBear<24>,
            Poseidon2InternalLayerKoalaBear<24>,
            24,
            3,
        >;
        type MyHash = PaddingFreeSponge<Perm24, 24, 16, 8>;
        type MyCompress = TruncatedPermutation<Perm16, 2, 8, 16>;
        type ValMmcs =
            MerkleTreeMmcs<<Val as Field>::Packing, <Val as Field>::Packing, MyHash, MyCompress, 8>;
        type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;
        type Challenger = DuplexChallenger<Val, Perm24, 24, 16>;
        type PCS = TwoAdicFriPcs<Val, Dft, ValMmcs, ChallengeMmcs>;
        type MyConfig = StarkConfig<PCS, Challenge, Challenger>;

        let env_filter = EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy();

        Registry::default()
            .with(env_filter)
            .with(ForestLayer::default())
            .init();

        let mut rng = SmallRng::seed_from_u64(1);

        let dft = Dft::default();

        let perm16 = Perm16::new_from_rng_128(&mut rng);
        let perm24 = Perm24::new_from_rng_128(&mut rng);

        let hash = MyHash::new(perm24.clone());
        let compress = MyCompress::new(perm16.clone());

        let val_mmcs = ValMmcs::new(hash, compress);

        let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());

        let fri_config = FriConfig {
            log_blowup,
            log_final_poly_len: 0,
            num_queries,
            proof_of_work_bits: 16,
            mmcs: challenge_mmcs,
        };

        // Generate a random trace matrix
        let trace = RowMajorMatrix::rand(&mut rng, 1 << LOG_TRACE_ROWS, 1 << LOG_TRACE_COLUMNS);
        println!(
            "trace dimension: rows={:?}, columns={:?}",
            trace.values.len() / trace.width,
            trace.width
        );
        let pcs = TwoAdicFriPcs::new(dft, val_mmcs, fri_config);

        let config = MyConfig::new(pcs);

        let mut proof_challenger = Challenger::new(perm24.clone());
        let mut verif_challenger = Challenger::new(perm24);

        let proof = prove_random_trace(&config, &mut proof_challenger, trace);
        report_proof_size_example(&proof);

        let result = verify_random_trace(&config, &mut verif_challenger, &proof);

        if let Err(e) = result {
            panic!("{:?}", e);
        } else {
            println!("Proof Verified Successfully")
        }
    }
    #[test]
    fn test_fri_inv_rate2() {
        println!("Test: FRI invRate = 2");
        test_fri(1, 256);
    }
}
