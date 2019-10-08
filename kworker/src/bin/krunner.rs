extern crate kworker;
extern crate env_logger;

use kworker::k;
use std::path::{PathBuf};

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let benchmarkpath = PathBuf::from("/home/sbugrara/k-distributed/tmp/0-simple00-0.4.24-26b56fe1-8bea-474b-b38d-36f9519558df/generated");
    let specname = "fn-execute-spec.k";

    k::run(&benchmarkpath, &specname, &"kprove", &"evm-semantics", Some(1800), Some(10024));
}