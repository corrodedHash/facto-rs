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

pub fn cargo_target_directory() -> Option<std::path::PathBuf> {
    #[derive(serde::Deserialize)]
    struct Metadata {
        target_directory: std::path::PathBuf,
    }

    let output = std::process::Command::new(std::env::var_os("CARGO")?)
        .args(&["metadata", "--format-version", "1"])
        .output()
        .ok()?;
    let metadata: Metadata = serde_json::from_slice(&output.stdout).ok()?;
    Some(metadata.target_directory)
}
