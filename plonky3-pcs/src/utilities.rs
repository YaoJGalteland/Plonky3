use std::fmt::Debug;

use p3_challenger::CanSample;
use p3_commit::{Pcs, PolynomialSpace};
use p3_matrix::Matrix;
use p3_matrix::dense::RowMajorMatrix;
use p3_uni_stark::{PcsError, StarkGenericConfig, Val, VerificationError};
use p3_util::log2_strict_usize;
use serde::{Deserialize, Serialize};

/// Type alias for commitment representation within a Stark proof.
type Com<SC> = <<SC as StarkGenericConfig>::Pcs as Pcs<
    <SC as StarkGenericConfig>::Challenge,
    <SC as StarkGenericConfig>::Challenger,
>>::Commitment;

/// Type alias for proof representation within a Stark proof.
type PcsProof<SC> = <<SC as StarkGenericConfig>::Pcs as Pcs<
    <SC as StarkGenericConfig>::Challenge,
    <SC as StarkGenericConfig>::Challenger,
>>::Proof;

/// Represents a proof structure for a random trace, without constraints and quotient polynomials.
#[derive(Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Proof<SC: StarkGenericConfig> {
    pub commitments: CommitmentsWithoutQuotient<Com<SC>>,
    pub opened_values: OpenedValues<SC::Challenge>,
    pub opening_proof: PcsProof<SC>,
    pub degree_bits: usize,
}

/// Stores commitments without quotient polynomials.
#[derive(Debug, Serialize, Deserialize)]
pub struct CommitmentsWithoutQuotient<Com> {
    pub trace: Com,
}

/// Stores values opened during proof verification.
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenedValues<Challenge> {
    pub trace_local: Vec<Challenge>,
    pub trace_next: Vec<Challenge>,
}

/// Report the size of the serialized proof.
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

/// Generates a STARK proof for a random trace.
pub fn prove_random_trace<SC>(
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

    let (trace_commit, trace_data) = pcs.commit(vec![(trace_domain, trace)]);

    let commitments = CommitmentsWithoutQuotient {
        trace: trace_commit,
    };

    let zeta: SC::Challenge = challenger.sample();
    let zeta_next = trace_domain.next_point(zeta).unwrap();

    let (opened_values, opening_proof) =
        pcs.open(vec![(&trace_data, vec![vec![zeta, zeta_next]])], challenger);
    let trace_local = opened_values[0][0][0].clone();
    let trace_next = opened_values[0][0][1].clone();
    let opened_values = OpenedValues {
        trace_local,
        trace_next,
    };

    Proof {
        commitments,
        opened_values,
        opening_proof,
        degree_bits: log_degree,
    }
}

/// Verifies a STARK proof for a random tracee.
pub fn verify_random_trace<SC>(
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

    let _result = pcs.verify(
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
    );

    Ok(())
}
