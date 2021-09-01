use facto::Factoring;
use std::io::Write;

use crate::bench_logger::{Bench, BenchmarkElement};

const DIGITS_OF_PI_1: u128 = 0xfc24_1156_c5a1_cb10_98c8_7451_0ec0_22ce;
const DIGITS_OF_PI_2: u128 = 0x9025_441c_f994_0b8c_7410_195f_7237_9136;
const DIGITS_OF_PI_3: u128 = 0x14a2_6784_0b1c_eec8_2a7d_f366_972c_3630;
const DIGITS_OF_PI_4_1: u64 = 0x3503_ed94_3e93_bb0a;
const DIGITS_OF_PI_4_2: u64 = 0x21fc_adab_e2ad_e890;
const DIGITS_OF_PI_5_1: u64 = 0x6849_9bab_9e91_d8df;
// const DIGITS_OF_PI_5_2: u64 = 0x8c8b_65d0_fb19_35ad;

struct U128RNG {
    a_state: u128,
}
impl U128RNG {
    const A_START: u128 = DIGITS_OF_PI_1;
    const A_FACTOR: u128 = DIGITS_OF_PI_2 + 3;
    const A_INCREMENT: u128 = DIGITS_OF_PI_3 + 1;

    fn new() -> Self {
        Self {
            a_state: Self::A_START,
        }
    }
}

impl Iterator for U128RNG {
    type Item = u128;

    fn next(&mut self) -> Option<Self::Item> {
        self.a_state = self
            .a_state
            .wrapping_mul(Self::A_FACTOR)
            .wrapping_add(Self::A_INCREMENT);
        Some(self.a_state)
    }
}
struct U64RNG {
    a_state: u64,
}
impl U64RNG {
    const A_START: u64 = DIGITS_OF_PI_4_1;
    const A_FACTOR: u64 = DIGITS_OF_PI_4_2 + 3;
    const A_INCREMENT: u64 = DIGITS_OF_PI_5_1;

    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            a_state: Self::A_START,
        }
    }
}

impl Iterator for U64RNG {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        self.a_state = self
            .a_state
            .wrapping_mul(Self::A_FACTOR)
            .wrapping_add(Self::A_INCREMENT);
        Some(self.a_state)
    }
}

#[test]
fn test_sequence_uniqueness() {
    let mut hits = vec![];
    hits.reserve(10_000);
    for (index, i) in (0..10_000).zip(U128RNG::new()) {
        if index % 10 == 0 {
            println!("{}", index);
        }
        match hits.binary_search(&i) {
            Ok(_) => panic!("Same value: {}", i),
            Err(pos) => hits.insert(pos, i),
        }
    }
}

#[allow(dead_code)]
fn bench_u64(csv_output: &mut std::fs::File) {
    const BENCH_COUNT: i32 = 100_000;
    writeln!(csv_output, ",\n\t\"results_u64\": [").expect("Could not write to file");
    for (index, n) in (0..BENCH_COUNT).zip(U64RNG::new()) {
        let start_time = std::time::SystemTime::now();
        let f = n.factor();
        let elapsed = start_time.elapsed().unwrap().as_micros();
        let formatted_factor_list = f
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(
            csv_output,
            "\t\t[ {}, {}, {}, [{}] ]{}",
            index,
            n,
            elapsed,
            &formatted_factor_list,
            if index < BENCH_COUNT - 1 { "," } else { "" }
        )
        .expect("Could not write to file");
        println!(
            "{:>7} [ {:>6}ms ]: {:>39} -> {}",
            index,
            elapsed / 1000,
            n,
            &formatted_factor_list
        );
    }
    write!(csv_output, "\t]").expect("Could not write to file");
}

pub struct RandomU128();
impl Bench for RandomU128 {
    fn name() -> String {
        "random_u128".to_string()
    }
    fn generate_elements() -> Vec<BenchmarkElement> {
        const BENCH_COUNT: i32 = 100;
        let mut results = vec![];
        results.reserve(BENCH_COUNT as usize);
        for (_, n) in (0..BENCH_COUNT).zip(U128RNG::new()) {
            let start_time = std::time::SystemTime::now();
            n.factor();
            let elapsed = start_time.elapsed().unwrap().as_micros();
            // let formatted_factor_list = f
            //     .iter()
            //     .map(|x| x.to_string())
            //     .collect::<Vec<_>>()
            //     .join(", ");
            // println!(
            //     "{:>7} [ {:>6}ms ]: {:>39} -> {}",
            //     index,
            //     elapsed / 1000,
            //     n,
            //     &formatted_factor_list
            // );
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            results.push(BenchmarkElement {
                n: n.to_string(),
                elapsed_microseconds: elapsed as u64,
            });
        }
        println!();
        results
    }
}
