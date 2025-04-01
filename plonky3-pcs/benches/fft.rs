use criterion::measurement::Measurement;
use criterion::{BenchmarkGroup, BenchmarkId, Criterion, criterion_group, criterion_main};
use p3_koala_bear::KoalaBear;
use p3_circle::{CircleDomain, CircleEvaluations};
use p3_dft::{Radix2Bowers, Radix2Dit, Radix2DitParallel, TwoAdicSubgroupDft};
use p3_field::TwoAdicField;
use p3_matrix::dense::RowMajorMatrix;
use p3_mersenne_31::Mersenne31;
use p3_util::pretty_name;
use rand::distr::{Distribution, StandardUniform};
use rand::rng;
use p3_baby_bear::BabyBear;
const LOG_N: usize = 19;
const LOG_W: usize = 11;

fn bench_lde_baby_bear(c: &mut Criterion) {
    let log_n = LOG_N;
    let log_w = LOG_W;

    let mut g = c.benchmark_group("lde");
    g.sample_size(10);
    lde_cfft(&mut g, log_n, log_w);
    lde_twoadic::<BabyBear, Radix2Dit<_>, _>(&mut g, log_n, log_w);
    lde_twoadic::<BabyBear, Radix2DitParallel<_>, _>(&mut g, log_n, log_w);
    lde_twoadic::<BabyBear, Radix2Bowers, _>(&mut g, log_n, log_w);
}
fn bench_lde_koala_bear(c: &mut Criterion) {
    let log_n = LOG_N;
    let log_w = LOG_W;

    let mut g = c.benchmark_group("lde");
    g.sample_size(10);
    lde_cfft(&mut g, log_n, log_w);
    lde_twoadic::<KoalaBear, Radix2Dit<_>, _>(&mut g, log_n, log_w);
    lde_twoadic::<KoalaBear, Radix2DitParallel<_>, _>(&mut g, log_n, log_w);
    lde_twoadic::<KoalaBear, Radix2Bowers, _>(&mut g, log_n, log_w);
}
fn lde_cfft<M: Measurement>(g: &mut BenchmarkGroup<M>, log_n: usize, log_w: usize) {
    type F = Mersenne31;
    let log_n = LOG_N;
    let log_w = LOG_W;
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
                    evals.extrapolate(CircleDomain::standard(log_n + 1))
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
                |(dft, m)| dft.coset_lde_batch(m, 1, F::GENERATOR),
                criterion::BatchSize::LargeInput,
            )
        },
    );
}

criterion_group!(benches, bench_lde_baby_bear, bench_lde_koala_bear);
criterion_main!(benches);
