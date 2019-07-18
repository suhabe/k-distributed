extern crate kworker;
extern crate walkdir;
#[macro_use] extern crate log;
extern crate env_logger;

use kworker::db::{exec};
use kworker::job::*;
use kworker::s3::*;
use rusoto_core::Region;
use rusoto_s3::{S3Client};
use std::env;
use std::path::{Path, PathBuf};
use std::fs;
use std::time;
use std::thread::{sleep};
use uuid::Uuid;


fn main() {
    env_logger::init();

    let client = S3Client::new(Region::UsEast2);
    let bucket_name = String::from("kjob");

    while (true) {
        match (exec(pop_job)) {
            None => {
                info!("No unprocessed jobs found. Sleeping...");
                exec(list_jobs);
                sleep(time::Duration::from_millis(3000))
            },

            Some(job_id) => {
                info!("Found job: {}", job_id);
            },
        }
    }

    //let client = S3Client::new(Region::UsEast2);
    //let bucket_name = String::from("kjob");

    /*let local_dir_path = String::from("/home/sbugrara/kevm-verify-benchmarks/0-simple00-0.4.24");
    let key_prefix = String::from("0-simple00-0.4.24");
    let key_root_dir = format!("{}-{}", key_prefix, Uuid::new_v4());

    s3_upload_dir(&client, &bucket_name, &key_root_dir, &local_dir_path);*/

    /*
    let key_root_dir = String::from("0-simple00-0.4.24-44553a4d-ac2a-4f17-96b8-fbba8633c18e");
    let dest_dir = String::from("/tmp");
    s3_download_dir(&client, &bucket_name, &key_root_dir, &dest_dir);*/
}
