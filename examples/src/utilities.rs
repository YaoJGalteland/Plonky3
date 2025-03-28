use std::fmt::Debug;
use rand::distr::{Distribution, StandardUniform};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use tracing::{info_span, instrument};
use p3_challenger::{CanSample, DuplexChallenger};
use p3_commit::{ExtensionMmcs, Pcs, PolynomialSpace};
use p3_dft::TwoAdicSubgroupDft;
use p3_field::{ExtensionField, PrimeField32, TwoAdicField};
use p3_fri::{FriConfig, TwoAdicFriPcs};
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;
use p3_symmetric::CryptographicPermutation;
use p3_uni_stark::{PcsError, StarkConfig, StarkGenericConfig, Val, VerificationError};
use p3_util::log2_strict_usize;
use crate::proofs::get_poseidon2_mmcs;

type Com<SC> = <<SC as StarkGenericConfig>::Pcs as Pcs<
    <SC as StarkGenericConfig>::Challenge,
    <SC as StarkGenericConfig>::Challenger,
>>::Commitment;
type PcsProof<SC> = <<SC as StarkGenericConfig>::Pcs as Pcs<
    <SC as StarkGenericConfig>::Challenge,
    <SC as StarkGenericConfig>::Challenger,
>>::Proof;

#[derive(Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Proof<SC: StarkGenericConfig> {
    pub commitments: CommitmentsWithoutQuotient<Com<SC>>,
    pub opened_values: OpenedValues<SC::Challenge>,
    pub opening_proof: PcsProof<SC>,
    pub degree_bits: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitmentsWithoutQuotient<Com> {
    pub trace: Com,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenedValues<Challenge> {
    pub trace_local: Vec<Challenge>,
    pub trace_next: Vec<Challenge>,
    pub(crate) quotient_chunks: Vec<Vec<Challenge>>,
}
/// Report the size of the serialized proof.
///
/// Serializes the given proof instance using bincode and prints the size in bytes.
/// Panics if serialization fails.
#[inline]
pub fn report_proof_size_example<SC>(proof: &Proof<SC>)
    where
        SC: StarkGenericConfig,
{
    let config = bincode::config::standard()
        .with_little_endian()
        .with_fixed_int_encoding();
    let proof_bytes =
        bincode::serde::encode_to_vec(proof, config).expect("Failed to serialize proof");
    println!("Proof size: {} bytes", proof_bytes.len());
}

pub fn prove_without_air<SC>(
    config: &SC,
    challenger: &mut SC::Challenger,
    trace: RowMajorMatrix<Val<SC>>,
) -> Proof<SC>
    where
        SC: StarkGenericConfig,
{
    let degree = trace.height();
    let log_degree = log2_strict_usize(degree);

    let pcs = config.pcs();
    let trace_domain = pcs.natural_domain_for_degree(degree);

    let (trace_commit, trace_data) =
        info_span!("commit to trace data").in_scope(|| pcs.commit(vec![(trace_domain, trace)]));

    let commitments = CommitmentsWithoutQuotient {
        trace: trace_commit,
    };

    let zeta: SC::Challenge = challenger.sample();
    let zeta_next = trace_domain.next_point(zeta).unwrap();

    let (opened_values, opening_proof) = info_span!("open")
        .in_scope(|| pcs.open(vec![(&trace_data, vec![vec![zeta, zeta_next]])], challenger));
    let trace_local = opened_values[0][0][0].clone();
    let trace_next = opened_values[0][0][1].clone();
    let opened_values = OpenedValues {
        trace_local,
        trace_next,
        quotient_chunks: vec![],
    };

    Proof {
        commitments,
        opened_values,
        opening_proof,
        degree_bits: log_degree,
    }
}

#[instrument(skip_all)]
pub fn verify_without_air<SC>(
    config: &SC,
    challenger: &mut SC::Challenger,
    proof: &Proof<SC>,
) -> Result<(), VerificationError<PcsError<SC>>>
    where
        SC: StarkGenericConfig,
{
    let Proof {
        commitments,
        opened_values,
        opening_proof,
        degree_bits,
    } = proof;

    let degree = 1 << degree_bits;

    let pcs = config.pcs();
    let trace_domain = pcs.natural_domain_for_degree(degree);

    let zeta: SC::Challenge = challenger.sample();
    let zeta_next = trace_domain.next_point(zeta).unwrap();

    pcs.verify(
        vec![(
            commitments.trace.clone(),
            vec![(
                trace_domain,
                vec![
                    (zeta, opened_values.trace_local.clone()),
                    (zeta_next, opened_values.trace_next.clone()),
                ],
            )],
        )],
        opening_proof,
        challenger,
    )
        .map_err(VerificationError::InvalidOpeningArgument)?;

    Ok(())
}
// Constants for trace matrix size
const LOG_TRACE_ROWS: usize = 19;
const LOG_TRACE_COLUMNS: usize = 11;

#[inline]
pub fn prove_pcs<
    F: PrimeField32 + TwoAdicField,
    EF: ExtensionField<F> + TwoAdicField,
    DFT: TwoAdicSubgroupDft<F>,
    Perm16: CryptographicPermutation<[F; 16]> + CryptographicPermutation<[F::Packing; 16]>,
    Perm24: CryptographicPermutation<[F; 24]> + CryptographicPermutation<[F::Packing; 24]>,
>(
    dft: DFT,
    perm16: Perm16,
    perm24: Perm24,
) -> Result<(), impl Debug>
    where
        StandardUniform: Distribution<F>,
{
    let val_mmcs = get_poseidon2_mmcs::<F, _, _>(perm16, perm24.clone());

    let challenge_mmcs = ExtensionMmcs::<F, EF, _>::new(val_mmcs.clone());
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

    let mut proof_challenger: DuplexChallenger<F, Perm24, 24, 16> =
        DuplexChallenger::new(perm24.clone());
    let mut verif_challenger: DuplexChallenger<F, Perm24, 24, 16> = DuplexChallenger::new(perm24);


    let proof = prove_without_air(&config, &mut proof_challenger, trace);
    report_proof_size_example(&proof);

    verify_without_air(&config, &mut verif_challenger, &proof)
}