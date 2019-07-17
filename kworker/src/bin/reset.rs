extern crate kworker;

use kworker::db::exec;
use kworker::job::*;

fn main() {
    exec(reset_job);
    exec(list_jobs);
}