extern crate wait_timeout;

use wait_timeout::ChildExt;
use std::process::{Command};
use std::ops::Add;
use std::path::{Path,PathBuf};
use std::time::{Duration, Instant};
use std::fs::File;
use std::fs;
use std::io::{BufReader, BufRead};

#[derive(Debug)]
pub struct KResult {
    pub output_file_path: Option<PathBuf>,
    pub error_file_path: Option<PathBuf>,
    pub status_code: Option<i32>,
    pub timed_out: bool,
    pub proved: Option<bool>
}

pub fn run(benchmarkpath: &Path, specname: &str, kpath: &str, sempath: &str, timeout: Option<i32>, memlimit_mb: Option<i32>) -> KResult {
    info!("Running kprove: {:?} {}", benchmarkpath, specname);

    let mut output_file_path = benchmarkpath.to_path_buf();
    output_file_path.push("out.txt");
    info!("Output file: {:?}", &output_file_path);
    let _ = fs::remove_file(&output_file_path);

    let mut error_file_path = benchmarkpath.to_path_buf();
    error_file_path.push("err.txt");
    info!("Error file: {:?}", &error_file_path);
    let _ = fs::remove_file(&error_file_path);

    let javapath = std::env::var("APP_JAVA_PATH").unwrap();
    //let kpath = std::env::var("APP_K_PATH").unwrap();
    //let sempath = std::env::var("APP_SEMANTICS_PATH").unwrap();

    let cppath = String::from(kpath).add("/target/release/k/lib/java/*");

    let smtpreludepath = Path::new(benchmarkpath).join("evm.smt2");
    let smtpreludepath = smtpreludepath.to_str().unwrap();

    let specpath = Path::new(benchmarkpath).join(specname);
    let specpath = specpath.to_str().unwrap();

    let mut args = Vec::new();
    args.push("-Dfile.encoding=UTF-8");
    args.push("-Djava.awt.headless=true");
    let mut ms;
    let mut mx;
    if let Some(v) = memlimit_mb {
        ms = format!("-Xms{}m", v);
        mx = format!("-Xmx{}m", v);
        args.push(&ms);
        args.push(&mx);
    }
    args.push("-Xss32m");
    args.push("-XX:+TieredCompilation");
    args.push("-ea");
    args.push("-cp");
    args.push(&cppath);
    args.push("org.kframework.main.Main");
    args.push("-kprove");
    args.push("-v");
    args.push("--debug");
    args.push("-d");
    args.push(&sempath);
    args.push("-m");
    args.push("VERIFICATION");
    args.push("--z3-impl-timeout");
    args.push("500");
    args.push("--deterministic-functions");
    args.push("--no-exc-wrap");
    args.push("--cache-func-optimized");
    args.push("--no-alpha-renaming");
    args.push("--format-failures");
    args.push("--boundary-cells");
    args.push("k,pc");
    args.push("--log-cells");
    args.push("k,output,statusCode,localMem,pc,gas,wordStack,callData,accounts,memoryUsed,#pc,#result,#target");
    args.push("--smt-prelude");
    args.push(smtpreludepath);
    args.push(specpath);

    let now = Instant::now();
    info!("Executing K prover: {} {:?}", javapath, &args.join(" "));


    let mut child = Command::new(javapath)
        .args(&args)
        .stdout(File::create(&output_file_path).unwrap())
        .stderr(File::create(&error_file_path).unwrap())
        .spawn()
        .expect("Failed to execute process");

    let status_code = match timeout {
        Some(timeout) => {
            let timeout = Duration::from_secs(timeout as u64);
            match child.wait_timeout(timeout).unwrap() {
                Some(status) => status.code(),
                None => {
                    info!("Child hasn't exited yet. Send kill signal");
                    child.kill().expect("child.kill returned error");
                    let wait_result = child.wait().expect("child.wait returned error");
                    wait_result.code()
                }
            }
        },
        None => {
            let wait_result = child.wait().expect("child.wait returned error");
            wait_result.code()
        }
    };
    info!("Terminated K prover: {:?} {:?}", status_code, now.elapsed().as_secs());


    let fout = File::open(&output_file_path).expect("Output file not found");
    let reader = BufReader::new(fout);
    let lines = reader.lines();
    let mut proved = None;
    for ol in lines {
        let l = ol.unwrap();
        if l.trim() == "#True" {
            proved = Some(true);
        }
        if l.trim() == "false" {
            proved = Some(false);
        }
    }

    let res = KResult {
        output_file_path: Some(output_file_path),
        error_file_path: Some(error_file_path),
        status_code,
        timed_out: status_code.is_none(),
        proved
    };

    return res;
}