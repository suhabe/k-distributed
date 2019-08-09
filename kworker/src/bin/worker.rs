extern crate kworker;
extern crate walkdir;
extern crate log;
extern crate env_logger;

use kworker::db::{exec};
use kworker::job::*;
use kworker::s3::*;
use kworker::k;
use rusoto_core::Region;
use rusoto_s3::{S3Client};
use std::time;
use std::thread::{sleep};
use log::{info};
use std::path::PathBuf;
use std::ops::Add;

fn run(job_id: i32) {
    let job = exec(|tx| {get_job(tx, job_id)});
    let client = S3Client::new(Region::UsEast2);
    let dest_dir = std::env::var("APP_WORKER_DIR").unwrap();

    let s3_bucket = job.s3_bucket.unwrap().to_owned();
    let s3_key = job.s3_key.unwrap().to_owned();

    let down_dir = s3_download_dir(&client, &s3_bucket, &s3_key, |x| true, &dest_dir);

    let mut gen_dir = down_dir.clone();
    gen_dir.push("generated");

    let apppath = std::env::var("APP_PATH").unwrap();
    let kpath = String::from(apppath.to_owned()).add("/").add(&job.kprove).add("/").add("k-distribution");
    let sempath = String::from(apppath.to_owned()).add("/").add(&job.semantics).add("/").add(".build/java");

    info!("KPATH: {}", kpath);
    info!("SEMPATH: {}", sempath);

    let kres = k::run(gen_dir.as_path(), &job.spec_filename, &kpath, &sempath, job.timeout_sec);

    let output_key = match kres.output_file_path {
        Some(ref p) => upload_log(&client, &s3_bucket, &s3_key, p),
        None => String::from("")
    };

    let error_key = match kres.error_file_path {
        Some(ref p) => upload_log(&client, &s3_bucket, &s3_key, p),
        None => String::from("")
    };

    exec(|tx| {complete_job(tx, job_id, &output_key, &error_key, kres.status_code, kres.timed_out, kres.proved)});
}

fn upload_log(client: &S3Client, s3_bucket: &String, s3_key: &String, p: &PathBuf) -> String {
    let filename = String::from(p.as_path().file_name().unwrap().to_str().unwrap());
    let key = format!("{}/generated/{}", s3_key, &filename);
    s3_upload_file(client, s3_bucket, &key, p);
    key
}

fn main() {
    dotenv::dotenv().ok();
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
