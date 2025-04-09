
use criterion::measurement::Measurement;
use criterion::{BenchmarkGroup, BenchmarkId, Criterion, criterion_group, criterion_main};
use p3_baby_bear::BabyBear;
use p3_circle::{CircleDomain, CircleEvaluations};
use p3_dft::{Radix2Bowers, Radix2Dit, Radix2DitParallel, TwoAdicSubgroupDft};
use p3_field::TwoAdicField;
use p3_koala_bear::KoalaBear;
use p3_matrix::dense::RowMajorMatrix;
use p3_mersenne_31::Mersenne31;
use p3_util::pretty_name;
use rand::distr::{Distribution, StandardUniform};
use rand::rng;

fn bench_lde_diff_flags(c: &mut Criterion) {

    for log_n in 18..22 {
        for log_w in 3..6 {
            let mut g = c.benchmark_group(format!("lde for different flags: log_n={},log_w={}", log_n,log_w));
            g.sample_size(10);
            lde_cfft(&mut g, log_n, log_w);
            lde_twoadic::<KoalaBear, Radix2Dit<_>, _>(&mut g, log_n, log_w);
            lde_twoadic::<KoalaBear, Radix2DitParallel<_>, _>(&mut g, log_n, log_w);
            //lde_twoadic::<KoalaBear, Radix2Bowers, _>(&mut g, log_n, log_w);
        }
    }


}

fn lde_cfft<M: Measurement>(g: &mut BenchmarkGroup<M>, log_n: usize, log_w: usize) {
    type F = Mersenne31;
    let m = RowMajorMatrix::<F>::rand(&mut rng(), 1 << log_n, 1 << log_w);
    g.bench_with_input(
        BenchmarkId::new("Cfft<M31>", format!("log_n={log_n},log_w={log_w}")),
        &m,
        |b, m| {
            b.iter_batched(
                || m.clone(),
                |m| {
                    let evals =
                        CircleEvaluations::from_natural_order(CircleDomain::standard(log_n), m);
                    evals.extrapolate(CircleDomain::standard(log_n + 1)) // extension rate = 2, add 1 bit
                },
                criterion::BatchSize::LargeInput,
            )
        },
    );
}

fn lde_twoadic<F: TwoAdicField, Dft: TwoAdicSubgroupDft<F>, M: Measurement>(
    g: &mut BenchmarkGroup<M>,
    log_n: usize,
    log_w: usize,
) where
    StandardUniform: Distribution<F>,
{
    let dft = Dft::default();
    let m = RowMajorMatrix::<F>::rand(&mut rng(), 1 << log_n, 1 << log_w);
    g.bench_with_input(
        BenchmarkId::new(
            format!("{},{}", pretty_name::<F>(), pretty_name::<Dft>()),
            format!("log_n={log_n},log_w={log_w}"),
        ),
        &(dft, m),
        |b, (dft, m)| {
            b.iter_batched(
                || (dft.clone(), m.clone()),
                |(dft, m)| dft.coset_lde_batch(m, 1, F::GENERATOR), // extension rate = 2, add 1 bit
                criterion::BatchSize::LargeInput,
            )
        },
    );
}
criterion_group!(benches_diff_flags, bench_lde_diff_flags);
//criterion_group!(benches_large_trace, bench_lde_large_trace);

// Conditionally compile the main function based on the enabled feature


// Conditionally compile the main function based on the enabled feature
//#[cfg(feature = "benches_diff_flags")]
criterion_main!(benches_diff_flags);

//criterion_main!(benches_large_trace);