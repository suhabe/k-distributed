extern crate kworker;
extern crate walkdir;
extern crate env_logger;

use kworker::db::{exec};
use kworker::job::*;
use kworker::s3::s3_upload_dir;
use rusoto_core::Region;
use rusoto_s3::{S3Client};
use std::env;
use std::path::{PathBuf};
use std::fs;
use uuid::Uuid;

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let args = env::args().collect::<Vec<String>>();
    let dirpath = PathBuf::from(&args[1]);
    let spec_filename = String::from(PathBuf::from(&args[2]).file_name().unwrap().to_str().unwrap());
    let kprove = &args[3];//String::from(args[3].to_owned());
    let semantics = &args[4];//String::from(args[4].to_owned());
    let timeout_sec = &args[6].parse::<i32>().unwrap();
    let memlimit_mb = &args[5].parse::<i32>().unwrap();

    let dirpathbuf = fs::canonicalize(&dirpath).unwrap();
    let benchmark_dir = String::from(dirpathbuf.as_path().to_str().unwrap());
    let benchmark_name = String::from(dirpath.file_name().unwrap().to_str().unwrap());
    let benchmark_key = format!("{}-{}", benchmark_name, Uuid::new_v4());
    //let job_name = format!("{}:{}", benchmark_name, spec_filename);

    let client = S3Client::new(Region::UsEast2);
    let bucket_name = String::from("kjob");

    s3_upload_dir(&client, &bucket_name, &benchmark_key, &benchmark_dir);

    exec(|trans| { new_job(trans, &benchmark_name, &spec_filename, &kprove, &semantics, &bucket_name, &benchmark_key, &spec_filename, *timeout_sec, *memlimit_mb) } );

    //exec(list_jobs);
}
