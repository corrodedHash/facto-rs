#![allow(dead_code)]

use std::{io::Write, time::SystemTime};

use bench_logger::BenchmarkGroup;

use crate::bench_logger::{Bench, BenchmarkElement, BenchmarkReport};
mod bench_logger;
mod bench_random;
mod bench_rug;
mod bench_u128;
mod util;

macro_rules! benchibench {
    ($($b:ty),+ $(,)?) => {{
        let mut report = BenchmarkReport::new();

        $(
            println!("{}", <$b>::name());
            report.reports.push(<$b>::bench());
        )+
        report
    }};
}

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

fn compare(a_path: &str, b_path: &str) {
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

fn replicate_factor(bench_path: &str) {
    let a = std::fs::OpenOptions::new()
        .read(true)
        .open(bench_path)
        .expect("Could not open benchmark file");
    let report: BenchmarkReport =
        serde_json::from_reader(a).expect("Could not parse benchmark file");
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&format!(
            "benchmark_factor_{:0>20}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ))
        .expect("Could not open file");
    let mut new_report = BenchmarkReport::new();
    for a in &report.reports {
        let mut new_group = BenchmarkGroup {
            benchmarks: vec![],
            name: a.name.clone(),
        };
        println!("{}", a.name);
        for e in &a.benchmarks {
            println!("\t{}", e.n);
            let t = SystemTime::now();
            std::process::Command::new("factor")
                .arg(&e.n)
                .output()
                .unwrap();
            let d = t.elapsed().unwrap();
            new_group.benchmarks.push(BenchmarkElement::new(&e.n, &d));
        }
        new_report.reports.push(new_group);
    }
    let j = serde_json::to_string_pretty(&new_report).unwrap();
    if file.write_all(j.as_bytes()).is_err() {
        println!("Could not write file");
        println!("{}", j);
    };
}

fn create_bench() {
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&format!(
            "benchmark_{:0>20}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ))
        .expect("Could not open file");

    let report = benchibench!(
        bench_u128::HugeSemiPrime,
        bench_u128::SemiPrimesU128,
        bench_u128::BatchFactorization,
        bench_random::RandomU128,
        bench_rug::DifficultCompositeRug,
        bench_rug::ManyCompositesRug,
    );

    let j = serde_json::to_string_pretty(&report).unwrap();
    if file.write_all(j.as_bytes()).is_err() {
        println!("Could not write file");
        println!("{}", j);
    };
}

fn main() {
    create_bench()
    // compare(
    //     "benchmark_00000000001632261963.json",
    //     "benchmark_factor_00000000001632262189.json",
    // )
}
