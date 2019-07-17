extern crate kworker;
extern crate walkdir;

use kworker::db::exec;
use kworker::job::*;

use rusoto_core::Region;
use rusoto_s3::{S3Client, S3, PutObjectRequest};
use uuid::Uuid;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::ffi::OsStr;
use walkdir::WalkDir;

fn main() {
    let client = S3Client::new(Region::UsEast2);
    let bucket_name = String::from("kjob");
    let local_dir_path = String::from("/home/sbugrara/kevm-verify-benchmarks/0-simple00-0.4.24");
    let key_prefix = String::from("0-simple00-0.4.24");
    let key_root_dir = format!("{}-{}", key_prefix, Uuid::new_v4());

    s3_upload_dir(&client, &bucket_name, key_root_dir, &local_dir_path);
}

fn s3_upload_dir(client: &S3Client, bucket_name: &String, key_root_dir: String, local_dir_path: &String) {
    let key_root_dir = Path::new(&key_root_dir);

    for entry in WalkDir::new(&local_dir_path).into_iter().filter_map(|e| e.ok()).filter(|e| e.path().is_file()) {
        let path = entry.path();
        let rel_path = path.strip_prefix(&local_dir_path).unwrap();
        let key_name = String::from(key_root_dir.join(rel_path).to_str().unwrap());
        println!("Uploading {}", key_name);

        let mut f = File::open(path).unwrap();
        let mut contents: Vec<u8> = Vec::new();
        f.read_to_end(&mut contents).unwrap();

        let req = PutObjectRequest {
            bucket: bucket_name.to_owned(),
            key: key_name,
            body: Some(contents.into()),
            acl: Some(String::from("public-read")),
            ..Default::default()
        };
        let result = client.put_object(req).sync().expect("Couldn't PUT object");
        println!("{:#?}", result);
    }
}

fn s3_download_dir(client: &S3Client, bucket_name: &String, key_root_dir: String, local_dir_path: &String) {

}