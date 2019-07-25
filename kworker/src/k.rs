extern crate wait_timeout;

use wait_timeout::ChildExt;
use std::process::{Command};
use std::ops::Add;
use std::path::{Path,PathBuf};
use std::time::{Duration, Instant};
use std::fs::File;
use std::fs;

#[derive(Debug)]
pub struct KResult {
    pub output_file_path: Option<PathBuf>,
    pub error_file_path: Option<PathBuf>,
    pub status_code: i32
}

pub fn run(benchmarkpath: &Path, specname: &str) -> KResult {
    info!("Running kprove: {:?} {}", benchmarkpath, specname);

    let mut output_file_path = benchmarkpath.to_path_buf();
    output_file_path.push("out.txt");
    info!("Output file: {:?}", &output_file_path);
    fs::remove_file(&output_file_path);

    let mut error_file_path = benchmarkpath.to_path_buf();
    error_file_path.push("err.txt");
    info!("Error file: {:?}", &error_file_path);
    fs::remove_file(&error_file_path);

    let javapath = std::env::var("APP_JAVA_PATH").unwrap();
    let kpath = std::env::var("APP_K_PATH").unwrap();
    let sempath = std::env::var("APP_SEMANTICS_PATH").unwrap();

    let cppath = String::from(kpath).add("/target/release/k/lib/java/*");

    let smtpreludepath = Path::new(benchmarkpath).join("evm.smt2");
    let smtpreludepath = smtpreludepath.to_str().unwrap();

    let specpath = Path::new(benchmarkpath).join(specname);
    let specpath = specpath.to_str().unwrap();

    let args = ["-Dfile.encoding=UTF-8",
        "-Djava.awt.headless=true",
        "-Xms1024m", "-Xmx8192m", "-Xss32m",
        "-XX:+TieredCompilation",
        "-ea",
        "-cp", &cppath,
        "org.kframework.main.Main",
        "-kprove",
        "-v",
        "--debug",
        "-d",
        &sempath,
        "-m", "VERIFICATION",
        "--z3-impl-timeout", "500",
        "--deterministic-functions",
        "--no-exc-wrap",
        "--cache-func-optimized",
        "--no-alpha-renaming",
        "--format-failures",
        "--boundary-cells", "k,pc",
        "--log-cells", "k,output,statusCode,localMem,pc,gas,wordStack,callData,accounts,memoryUsed,#pc,#result,#target",
        "--smt-prelude", smtpreludepath,
        specpath];


    let now = Instant::now();
    info!("Executing K prover: {} {:?}", javapath, &args.join(" "));


    let mut child = Command::new(javapath)
        .args(&args)
        .stdout(File::create(&output_file_path).unwrap())
        .stderr(File::create(&error_file_path).unwrap())
        .spawn()
        .expect("Failed to execute process");

    let one_sec = Duration::from_secs(600);
    let status_code = match child.wait_timeout(one_sec).unwrap() {
        Some(status) => status.code().unwrap(),
        None => {
            // child hasn't exited yet
            child.kill().unwrap();
            child.wait().unwrap().code().unwrap()
        }
    };
    info!("Terminated K prover: {:?} {:?}", status_code, now.elapsed().as_secs());

    let res = KResult {
        output_file_path: Some(output_file_path),
        error_file_path: Some(error_file_path),
        status_code
    };

    return res;
}