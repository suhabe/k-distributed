extern crate kworker;
extern crate env_logger;

use kworker::db::exec;
use kworker::job::*;

fn main() {
    env_logger::init();

    exec(list_jobs);
}