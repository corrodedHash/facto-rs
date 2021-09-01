#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkElement {
    pub n: String,
    pub elapsed_microseconds: u64,
}

impl BenchmarkElement {
    pub fn new<T: ToString>(n: &T, t: &std::time::Duration) -> Self {
        Self {
            n: n.to_string(),
            elapsed_microseconds: t.as_micros() as u64,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkGroup {
    pub name: String,
    pub benchmarks: Vec<BenchmarkElement>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkReport {
    pub commit: String,
    pub unix_timestamp: u64,
    pub dirty: bool,
    pub reports: Vec<BenchmarkGroup>,
}

impl BenchmarkReport {
    pub fn new() -> BenchmarkReport {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let git_clean = std::process::Command::new("git")
            .args(["diff", "--quiet"])
            .output()
            .expect("Could not run git")
            .status
            .success();
        let git_commit_bytes = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("Could not run git")
            .stdout;
        let git_commit =
            std::str::from_utf8(&git_commit_bytes[..git_commit_bytes.len() - 1]).unwrap();

        Self {
            commit: git_commit.to_owned(),
            unix_timestamp: timestamp,
            dirty: !git_clean,
            reports: vec![],
        }
    }
}

pub trait Bench {
    fn name() -> String;
    fn generate_elements() -> Vec<BenchmarkElement>;
    fn bench() -> BenchmarkGroup {
        BenchmarkGroup {
            benchmarks: Self::generate_elements(),
            name: Self::name(),
        }
    }
}
