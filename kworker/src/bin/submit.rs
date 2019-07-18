extern crate kworker;
extern crate walkdir;

use kworker::db::{exec, exec2};
use kworker::job::*;
use kworker::s3::*;
use rusoto_core::Region;
use rusoto_s3::{S3Client};
use std::env;
use std::path::{Path, PathBuf};
use std::fs;
use uuid::Uuid;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let dirpath = PathBuf::from(&args[1]);
    let dirpathbuf = fs::canonicalize(&dirpath).unwrap();
    let benchmark_dir = String::from(dirpathbuf.as_path().to_str().unwrap());
    let benchmark_name = String::from(dirpath.file_name().unwrap().to_str().unwrap());
    let benchmark_key = format!("{}-{}", benchmark_name, Uuid::new_v4());

    let client = S3Client::new(Region::UsEast2);
    let bucket_name = String::from("kjob");

   // s3_upload_dir(&client, &bucket_name, &benchmark_key, &benchmark_dir);

    exec2(|trans| { new_job(trans, &benchmark_name, &bucket_name, &benchmark_key, 1800) } );

    exec(list_jobs);

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
