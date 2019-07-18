extern crate kworker;
extern crate walkdir;
extern crate log;
extern crate env_logger;

use kworker::db::{exec};
use kworker::job::*;
use kworker::s3::*;
use rusoto_core::Region;
use rusoto_s3::{S3Client};
use std::time;
use std::thread::{sleep};
use log::{info};

fn run(job_id: i32) {
    let job = exec(|tx| {get_job(tx, job_id)});
    let client = S3Client::new(Region::frUsEast2);
    let dest_dir = std::env::var("APP_WORKER_DIR").unwrap();

    s3_download_dir(&client, &job.s3_bucket.unwrap(), &job.s3_key.unwrap(), &dest_dir);
}

fn main() {
    env_logger::init();

    loop {
        match exec(pop_job) {
            None => {
                info!("No unprocessed jobs found. Sleeping...");
                sleep(time::Duration::from_millis(3000))
            },

            Some(job_id) => {
                info!("Found job: {}", job_id);
                run(job_id);
            },
        }
    }
}
