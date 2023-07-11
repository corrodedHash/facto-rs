use crate::bench_logger::{BenchmarkGroup, BenchmarkReport};

fn log_delta(a: f64, b: f64) -> f64 {
    100f64 * (a / b).ln()
}

fn compare_benchmark_group(a: &BenchmarkGroup, b: &BenchmarkGroup) {
    let a_total = a
        .benchmarks
        .iter()
        .fold(0u64, |x, y| x + y.elapsed_microseconds);
    let b_total = b
        .benchmarks
        .iter()
        .fold(0u64, |x, y| x + y.elapsed_microseconds);

    println!(
        "{} {} {} {:7.1}",
        a.name,
        a_total,
        b_total,
        log_delta(b_total as f64, a_total as f64)
    );
    let mut lines = vec![];
    for (a_e, b_e) in a.benchmarks.iter().zip(&b.benchmarks) {
        assert_eq!(a_e.n, b_e.n);
        let change_factor = log_delta(
            b_e.elapsed_microseconds as f64,
            a_e.elapsed_microseconds as f64,
        );
        lines.push((
            a_e.n.clone(),
            a_e.elapsed_microseconds,
            b_e.elapsed_microseconds,
            change_factor,
        ));
    }
    lines.sort_unstable_by(|(_, _, _, x), (_, _, _, y)| {
        <f64 as PartialOrd>::partial_cmp(x, y).unwrap()
    });
    for (n, a, b, delta) in lines {
        println!("{:>50}: {:>8} {:>8} {:7.1}", n, a, b, delta)
    }
}

fn compare_benchmarks(a: &BenchmarkReport, b: &BenchmarkReport) {
    let mut a_unique_group_count = 0;
    for a_group in &a.reports {
        let b_group = b.reports.iter().find(|x| x.name == a_group.name);
        if let Some(b_group) = b_group {
            compare_benchmark_group(a_group, b_group);
        } else {
            a_unique_group_count += 1;
        }
    }
    let b_unique_group_count = b.reports.len() - (a.reports.len() - a_unique_group_count);
    if a_unique_group_count > 0 || b_unique_group_count > 0 {
        println!(
            "#Benchmarks in A not in B: {}\n#Benchmarks in B not in A: {}\n",
            a_unique_group_count, b_unique_group_count
        );
    }
}

pub fn compare(a_path: &str, b_path: &str) {
    let a = std::fs::OpenOptions::new()
        .read(true)
        .open(a_path)
        .expect("Could not open benchmark A");
    let b = std::fs::OpenOptions::new()
        .read(true)
        .open(b_path)
        .expect("Could not open benchmark B");
    let a: BenchmarkReport = serde_json::from_reader(a).unwrap();
    let b: BenchmarkReport = serde_json::from_reader(b).unwrap();
    compare_benchmarks(&a, &b);
}
