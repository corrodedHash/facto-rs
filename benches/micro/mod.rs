use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use facto::factoring::TrialDivision;
fn rug_overhead_u32(c: &mut Criterion) {
    let mut group = c.benchmark_group("u32");
    for i in [962089271u32, 1638840788, 1251399164, 782678445, 2876196663] {
        group.bench_with_input(
            BenchmarkId::new("rug", i),
            &rug::Integer::from(i),
            |b, i: &rug::Integer| b.iter(|| i.clone().exhaustive_trial_division()),
        );
        group.bench_with_input(BenchmarkId::new("native", i), &i, |b, i: &u32| {
            b.iter(|| i.exhaustive_trial_division())
        });
    }
    group.finish()
}
fn rug_overhead_u64(c: &mut Criterion) {
    let mut group = c.benchmark_group("u64");
    for i in [
        15013404555801482557u64,
        // 908113297543094557,
        2563631115007850458,
        18201360545243250159,
        // 427008916666980211,
    ] {
        group.bench_with_input(
            BenchmarkId::new("rug", i),
            &rug::Integer::from(i),
            |b, i: &rug::Integer| b.iter(|| i.clone().exhaustive_trial_division()),
        );
        group.bench_with_input(BenchmarkId::new("native", i), &i, |b, i: &u64| {
            b.iter(|| i.exhaustive_trial_division())
        });
    }
    group.finish()
}

criterion_group!(benches, rug_overhead_u32, rug_overhead_u64);
criterion_main!(benches);
