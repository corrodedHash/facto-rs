use facto::Factoring;

use crate::bench_logger::{Bench, BenchmarkElement};

pub struct DifficultCompositeRug();
impl Bench for DifficultCompositeRug {
    fn name() -> String {
        "difficult_composite_rug".to_string()
    }
    fn generate_elements() -> Vec<BenchmarkElement> {
        let prime_deltas = [
            // (47, vec![115, 127, 147]),
            (47, vec![115, 127, 147, 279, 297, 339, 435, 541, 619, 649]),
            // (48, vec![59, 65, 89, 93, 147, 165, 189, 233, 243, 257]),
            // (49, vec![81, 111, 123, 139, 181, 201, 213, 265, 283, 339]),
            // (50, vec![27, 35, 51, 71, 113, 117, 131, 161, 195, 233]),
            // (51, vec![129, 139, 165, 231, 237, 247, 355, 391, 397, 439]),
        ];
        let mut r = rug::Integer::from(1);
        for (e, primes) in prime_deltas {
            let power = 1u64 << e;
            for p in primes {
                r *= power - p;
            }
        }
        let r_bak = r.clone();
        let t = std::time::SystemTime::now();
        r.factor_events(crate::util::DottingEventSubscriptor());
        let d = t.elapsed().unwrap();
        println!();
        vec![BenchmarkElement::new(&r_bak, &d)]
    }
}

pub struct ManyCompositesRug();
impl Bench for ManyCompositesRug {
    fn name() -> String {
        "many_composites_rug".to_string()
    }

    fn generate_elements() -> Vec<BenchmarkElement> {
        let prime_deltas = [
            (18, vec![5, 11, 17, 23, 33, 35, 41, 65, 75, 93]),
            (19, vec![1, 19, 27, 31, 45, 57, 67, 69, 85, 87]),
            (20, vec![3, 5, 17, 27, 59, 69, 129, 143, 153, 185]),
            (21, vec![9, 19, 21, 55, 61, 69, 105, 111, 121, 129]),
        ];
        let mut r = rug::Integer::from(1);
        for (e, primes) in prime_deltas {
            let power = 1 << e;
            for p in primes {
                r *= power - p;
            }
        }
        let r_bak = r.clone();
        let t = std::time::SystemTime::now();
        r.factor_events(crate::util::DottingEventSubscriptor());
        let d = t.elapsed().unwrap();
        println!();
        vec![BenchmarkElement::new(&r_bak, &d)]
    }
}
