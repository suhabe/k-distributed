extern crate kworker;

use kworker::db::exec;
use kworker::job::*;

fn main() {
    exec(list_jobs);
}