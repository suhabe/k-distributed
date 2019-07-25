extern crate kworker;
extern crate env_logger;

use kworker::k;

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let benchmarkpath = "/home/sbugrara/k-distributed/tmp/0-simple00-0.4.24-26b56fe1-8bea-474b-b38d-36f9519558df/generated";
    let specname = "fn-execute-spec.k";

    k::run(&benchmarkpath, &specname);
}