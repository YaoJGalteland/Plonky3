use std::fmt::Debug;
use std::result;
use std::time::Instant;

use p3_baby_bear::BabyBear;
use p3_challenger::{HashChallenger, SerializingChallenger32, SerializingChallenger64};
use p3_commit::{ExtensionMmcs, Pcs};
use p3_field::extension::BinomialExtensionField;
use p3_fri::{TwoAdicFriPcs, create_benchmark_fri_config};
use p3_keccak_air::{KeccakAir, generate_trace_rows};
use p3_koala_bear::KoalaBear;
use p3_merkle_tree::MerkleTreeMmcs;
use p3_sha256::Sha256;
use p3_symmetric::{CompressionFunctionFromHasher, PaddingFreeSponge, SerializingHasher32, SerializingHasher32To64, SerializingHasher64};
use p3_uni_stark::{StarkConfig, prove, verify};
use rand::random;
use tracing::{info, info_span, Level};
use tracing_forest::ForestLayer;
use tracing_forest::util::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, FmtSubscriber, Registry};
use p3_dft::Radix2DitParallel;
use p3_examples::proofs::{get_keccak_mmcs, report_result};
use p3_keccak::{Keccak256Hash, KeccakF};

const TRACE_LENGTH: u8 = 19;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use regex::Regex;
use std::collections::HashMap;

fn main()  {

    // Log an event
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    Registry::default()
        .with(env_filter)
        .with(ForestLayer::default())
        .init();


    type Val = KoalaBear;

    let NUM_HASHES= (1 << TRACE_LENGTH) / 24;

    type Dft = Radix2DitParallel<Val>;
    let dft = Dft::default();

    /// Produce a MerkleTreeMmcs which uses the KeccakF permutation.
    let val_mmcs = get_keccak_mmcs();

    type Challenge = BinomialExtensionField<Val, 4>;
    let challenge_mmcs = ExtensionMmcs::<Val, Challenge, _>::new(val_mmcs.clone());

    type Challenger = SerializingChallenger64<Val, HashChallenger<u8, Keccak256Hash, 32>>;
    let fri_config = create_benchmark_fri_config(challenge_mmcs);

    let inputs = (0..NUM_HASHES).map(|_| random()).collect::<Vec<_>>();
    let trace = generate_trace_rows::<Val>(inputs, fri_config.log_blowup);
    println!("trace size: {:?} bits", trace.values.len().ilog2());
    println!("trace width: {:?} bits", trace.width.ilog2());

    //type Pcs = TwoAdicFriPcs<Val, Dft, ValMmcs, ChallengeMmcs>;
    let pcs = TwoAdicFriPcs::new(dft, val_mmcs, fri_config);

    //type MyConfig = StarkConfig<Pcs, Challenge, Challenger>;
    let config = StarkConfig::new(pcs);

    let mut challenger = Challenger::from_hasher(vec![], Keccak256Hash);
    let proof = prove(&config, &KeccakAir {}, &mut challenger, trace, &vec![]);

    let mut challenger = Challenger::from_hasher(vec![], Keccak256Hash);
    let result=verify(&config, &KeccakAir {}, &mut challenger, &proof, &vec![]);
    report_result(result);


}
