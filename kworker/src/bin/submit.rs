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
    env_logger::init();

    let args = env::args().collect::<Vec<String>>();
    let dirpath = PathBuf::from(&args[1]);
    let dirpathbuf = fs::canonicalize(&dirpath).unwrap();
    let benchmark_dir = String::from(dirpathbuf.as_path().to_str().unwrap());
    let benchmark_name = String::from(dirpath.file_name().unwrap().to_str().unwrap());
    let benchmark_key = format!("{}-{}", benchmark_name, Uuid::new_v4());

    let client = S3Client::new(Region::UsEast2);
    let bucket_name = String::from("kjob");

    s3_upload_dir(&client, &bucket_name, &benchmark_key, &benchmark_dir);

    exec(|trans| { new_job(trans, &benchmark_name, &bucket_name, &benchmark_key, 1800) } );

    exec(list_jobs);
}
