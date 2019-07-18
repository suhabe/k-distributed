use rusoto_core::Region;
use rusoto_s3::{S3Client, S3, PutObjectRequest, ListObjectsRequest, GetObjectRequest};
use uuid::Uuid;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::ffi::OsStr;
use walkdir::WalkDir;
use std::io::{Write, BufReader, BufRead, Error};


pub fn s3_upload_dir(client: &S3Client, bucket_name: &String, key_root_dir: &String, local_dir_path: &String) {
    let key_root_dir = Path::new(key_root_dir);

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

pub fn s3_download_dir(client: &S3Client, bucket_name: &String, key_root_dir: &String, dest_dir: &String) {

    let req = ListObjectsRequest {
        bucket: bucket_name.to_owned(),
        prefix: Some(key_root_dir.to_owned()),
        ..Default::default()
    };

    let objs = client.list_objects(req).sync().expect("Couldn't list key").contents.unwrap();

    let keys = objs.into_iter()
        .filter_map(|o| { o.key })
        .collect::<Vec<String>>();

    for key in keys {
        let path = Path::new(dest_dir).join(Path::new(&key));

        let full_key_pb = Path::new("/").join(Path::new(&bucket_name.to_owned()).join(Path::new(&key)));
        let full_key = full_key_pb.to_str().unwrap();

        std::fs::create_dir_all(path.parent().unwrap());

        let req = GetObjectRequest {
            bucket: bucket_name.to_owned(),
            key: key.to_owned(),
            ..Default::default()
        };
        let result = client.get_object(req).sync().expect("Couldn't GET object");
        let stream = result.body.unwrap();


        let mut buf = String::new();
        stream.into_blocking_read().read_to_string(&mut buf);

        let mut f = File::create(&path).unwrap();
        write!(f, "{}", buf);

        println!("Downloaded {:#?} {:#?}", &key, path.display());
    }

}