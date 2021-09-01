pub(crate) struct DottingEventSubscriptor();
impl<T> facto::FactoringEventSubscriptor<T> for DottingEventSubscriptor {
    fn factorized(&mut self, _n: &T, primes: &[T], _composites: &[T], _unknown: &[T]) {
        let mut bla = "".to_string();
        for _ in 0..primes.len() {
            bla += "."
        }
        print!(".");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }

    fn is_prime(&mut self, _n: &T) {
        print!(".");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }

    fn is_composite(&mut self, _n: &T) {}
}
