use rusoto_s3::{S3Client, S3, PutObjectRequest, ListObjectsRequest, GetObjectRequest};
use std::io::prelude::*;
use std::fs::File;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use std::io::{Write};

pub fn s3_url(bucket_name: &String, key_name: &String) -> String {
    format!("http://{}.s3.amazonaws.com/{}", bucket_name, key_name)
}

pub fn s3_url_opt(bucket_name: Option<String>, key_name: Option<String>) -> Option<String> {
    let b = bucket_name?;
    let k = key_name?;
    Some(s3_url(&b, &k))
}

pub fn s3_upload_dir(client: &S3Client, bucket_name: &String, key_root_dir: &String, local_dir_path: &String) {
    info!("Uploading to S3 {} {} {}", &bucket_name, &key_root_dir, &local_dir_path);

    let key_root_dir = Path::new(key_root_dir);

    for entry in WalkDir::new(&local_dir_path).into_iter().filter_map(|e| e.ok()).filter(|e| e.path().is_file()) {
        let path = entry.path();
        let rel_path = path.strip_prefix(&local_dir_path).unwrap();
        let key_name = String::from(key_root_dir.join(rel_path).to_str().unwrap());
        info!("\t{}", key_name);

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
        client.put_object(req).sync().expect("Couldn't PUT object");
    }
}

pub fn s3_upload_file(client: &S3Client, bucket_name: &String, bucket_key: &String, local_file_path: &PathBuf) {
    info!("Uploading to S3 {} {} {:?}", &bucket_name, &bucket_key, &local_file_path);

    let mut f = File::open(local_file_path).unwrap();
    let mut contents: Vec<u8> = Vec::new();
    f.read_to_end(&mut contents).unwrap();

    let req = PutObjectRequest {
        bucket: bucket_name.to_owned(),
        key: bucket_key.to_owned(),
        body: Some(contents.into()),
        acl: Some(String::from("public-read")),
        ..Default::default()
    };
    client.put_object(req).sync().expect("Couldn't PUT object");
}

pub fn s3_download_dir(client: &S3Client, bucket_name: &String, key_root_dir: &String, dest_dir: &String) -> PathBuf {
    info!("Downloading from S3 {} {} {}", &bucket_name, &key_root_dir, &dest_dir);

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
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();

        let req = GetObjectRequest {
            bucket: bucket_name.to_owned(),
            key: key.to_owned(),
            ..Default::default()
        };
        let result = client.get_object(req).sync().expect("Couldn't GET object");
        let stream = result.body.unwrap();


        let mut buf = String::new();
        stream.into_blocking_read().read_to_string(&mut buf).unwrap();

        let mut f = File::create(&path).unwrap();
        write!(f, "{}", buf).unwrap();

        println!("\t{:#?} {:#?}", &key, path.display());
    }

    let down_dir: PathBuf = [&dest_dir, &key_root_dir].iter().collect();

    down_dir
}