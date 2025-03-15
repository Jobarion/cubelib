use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

use cubelib::algs::Algorithm;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct LengthTestCase {
    scramble: String,
    config: String,
    length: isize,
    timeout_millis: u32
}

#[test]
fn run_length_tests() {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b'|')
        .from_path("tests/length_tests.csv")
        .unwrap();
    for result in reader.deserialize() {
        let record: LengthTestCase = result.expect("A CSV record");
        println!("Testing {} {} {}", record.scramble, record.config, record.length);
        run_length_test(&record, "iter-stream");
        run_length_test(&record, "multi-path-channel");
    }
}

fn run_length_test(test: &LengthTestCase, backend: &str) {
    let mut path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    path.push("target/release/cubelib-cli.exe");
    let output = Command::new(path.clone())
        .args(["--log", "error"])
        .arg("solve")
        .arg("--steps")
        .arg(test.config.as_str())
        .args(["--format", "compact"])
        .args(["--backend", backend])
        .args(["--quality", "0"])
        .args(["-n", "1"])
        .arg("--")
        .arg(test.scramble.as_str())
        .output()
        .expect("Failed to execute command");
    let alg_string = String::from_utf8(output.stdout).expect("Expected valid UTF-8");
    let alg_string = alg_string.trim().to_string();
    let parts = alg_string.rsplit_once("(");
    println!("Solution: {alg_string}");
    if let Some((alg_string, length)) = parts {
        let reported_length = usize::from_str(&length[0..(length.len() - 1)]).expect("Expected length number");
        let alg = Algorithm::from_str(alg_string).expect("Expected valid algorithm");
        assert_eq!(reported_length, alg.len());
        assert_eq!(test.length, alg.len() as isize);
    } else {
        if test.length != -1 {
            assert!(false, "No solution found")
        }
    }
    println!("Okay");
    println!();
}