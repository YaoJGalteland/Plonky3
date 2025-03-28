use std::fmt::Debug;

use p3_dft::{Radix2DitParallel};
use p3_examples::dfts::DftChoice;
use p3_examples::proofs::{
    report_result,
};
use p3_field::extension::BinomialExtensionField;
use p3_koala_bear::{KoalaBear, KoalaBearParameters, Poseidon2KoalaBear};

use rand::{rng};


use tracing_forest::ForestLayer;
use tracing_forest::util::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};
use p3_examples::utilities::{ prove_pcs};

use p3_monty_31::MontyField31;

fn main() {
    // Initializes logging
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    Registry::default()
        .with(env_filter)
        .with(ForestLayer::default())
        .init();

    type EF = BinomialExtensionField<KoalaBear, 4>;

    let dft: DftChoice<MontyField31<KoalaBearParameters>> = DftChoice::Parallel(Radix2DitParallel::default());

    let perm16 = Poseidon2KoalaBear::<16>::new_from_rng_128(&mut rng());
    let perm24 = Poseidon2KoalaBear::<24>::new_from_rng_128(&mut rng());

   let result = prove_pcs::<_, EF, _, _, _>(dft, perm16, perm24);
    report_result(result);
}
