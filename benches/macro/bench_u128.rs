use facto::Factoring;

pub fn black_box<T>(dummy: T) -> T {
    unsafe {
        let ret = std::ptr::read_volatile(&dummy);
        std::mem::forget(dummy);
        ret
    }
}

use crate::bench_logger::{Bench, BenchmarkElement};

pub struct SemiPrimesU128();
impl Bench for SemiPrimesU128 {
    fn name() -> String {
        "semi_primes_u128".to_string()
    }
    fn generate_elements() -> Vec<BenchmarkElement> {
        let prime_deltas = [
            (56, vec![5]),
            (55, vec![55]),
            (48, vec![59, 65, 89, 93]),
            (47, vec![115, 127, 147, 279]),
            // (26, vec![5, 27, 45, 87]),
            // (25, vec![39, 49, 61, 85]),
        ];
        let primes: Vec<u128> = prime_deltas
            .iter()
            .flat_map(|(e, d)| {
                let power = 1u128 << e;
                d.iter().map(|x| power - x).collect::<Vec<u128>>()
            })
            .collect();
        let mut composites = vec![];
        for i in 0..(primes.len() - 1) {
            for j in (i + 1)..(primes.len()) {
                composites.push(primes[i] * primes[j]);
            }
        }
        let mut results = vec![];
        for c in composites {
            let t = std::time::SystemTime::now();
            u128::factor(black_box(c));
            let d = t.elapsed().unwrap();
            results.push(BenchmarkElement::new(&c, &d));

            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
        println!();
        results
    }
}

pub struct HugeSemiPrime();
impl Bench for HugeSemiPrime {
    fn name() -> String {
        "huge_semi_prime".to_string()
    }
    fn generate_elements() -> Vec<BenchmarkElement> {
        const PRIME: u128 = ((1u128 << 63) - 25) * ((1u128 << 63) - 165);
        let t = std::time::SystemTime::now();
        // 85070591730234614113402964855534653469
        u128::factor(black_box(PRIME));
        let d = t.elapsed().unwrap();
        vec![BenchmarkElement::new(&PRIME, &d)]
    }
}
pub struct BatchFactorization();
impl Bench for BatchFactorization {
    fn name() -> String {
        "batch_factorization_u128".to_string()
    }
    fn generate_elements() -> Vec<BenchmarkElement> {
        // seq 1267650600228229401496703205376 1267650600228229401496703205476 | time factor
        // coreutils/factor: 0.773

        let mut result = vec![];
        for i in (0..=100).map(|x| x + (1u128 << 100)) {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();

            let x = std::time::SystemTime::now();
            u128::factor(black_box(i));
            let d = x.elapsed().unwrap();
            result.push(BenchmarkElement::new(&i, &d));
        }
        println!();
        result
    }
}
