#![allow(dead_code)]

use std::{io::Write, time::SystemTime};

use bench_logger::BenchmarkGroup;

use crate::bench_logger::{Bench, BenchmarkElement, BenchmarkReport};
mod bench_logger;
mod bench_random;
mod bench_rug;
mod bench_u128;
mod compare;
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
    let target_path =
        util::cargo_target_directory().expect("Could not determine crate root directory");
    let benchmark_dir_path = target_path.join("macro_bench");
    std::fs::create_dir_all(&benchmark_dir_path).expect("Could not create benchmark directory");

    let benchmark_filter = |x: &std::fs::DirEntry| {
        x.file_type().unwrap().is_file()
            && x.file_name()
                .into_string()
                .unwrap()
                .starts_with("benchmark_")
            && x.file_name().into_string().unwrap().ends_with(".json")
    };
    let last_benchmark_path = benchmark_dir_path
        .read_dir()
        .expect("Could not read benchmark directory")
        .filter_map(|x| x.ok())
        .filter(benchmark_filter)
        .max_by_key(|a| a.file_name())
        .map(|x| benchmark_dir_path.join(x.file_name()));

    let benchmark_path = benchmark_dir_path.join(format!(
        "benchmark_{:0>20}.json",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ));
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&benchmark_path)
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
    if let Some(lb) = last_benchmark_path {
        compare::compare(lb.to_str().unwrap(), benchmark_path.to_str().unwrap());
    }
}

fn main() {
    create_bench()
}
