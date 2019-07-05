extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_rds;
extern crate postgres;
extern crate uuid;

use rusoto_core::Region;
use rusoto_rds::{RdsClient, Rds};
use rusoto_s3::{S3Client, S3, PutObjectRequest};
use postgres::{Client, NoTls, Error};
use uuid::Uuid;
use std::fs::File;

fn main() {
  connect_db();
}

fn connect_s3() {
    let client = S3Client::new(Region::UsEast2);
    let bucket_name = "kjob";
    let benchmark_name = "0-simple00-0.4.24";
    let key_name = format!("{}-{}", benchmark_name, Uuid::new_v4());

    let mut f = File::open(local_filename).unwrap();
    let mut contents: Vec<u8> = Vec::new();
    match f.read_to_end(&mut contents) {
        Err(why) => panic!("Error opening file to send to S3: {}", why),
        Ok(_) => {
            let req = PutObjectRequest {
                bucket: bucket.to_owned(),
                key: dest_filename.to_owned(),
                body: Some(contents.into()),
                metadata: Some(metadata.clone()),
                ..Default::default()
            };
            let result = client.put_object(req).sync().expect("Couldn't PUT object");
            println!("{:#?}", result);
        }
    }

    s3_client.put_object().sync().expect("could not upload");
}

fn describe_db_instances() {
    let client = RdsClient::new(Region::UsEast2);
    match client.describe_db_instances(Default::default()).sync() {
        Ok(output) => {
            match output.db_instances {
                Some(instances) => {
                    println!("{:?}", instances);
                },
                None => ()
            }
        },
        Err(error) => {
            println!("Error: {:?}", error);
        },
    };
}

fn connect_db() {
    let hostname = "kprovedb2.cqfgjsgwdka2.us-east-2.rds.amazonaws.com";
    let username = "kuser";
    let password = "";

    let connectstr = format!("host={} user={}", hostname, username);

    let mut client = Client::configure()
         .host(hostname)
         .user(username)
         .password(password)
         .port(5432)
         .password(password)
         .connect(NoTls)?;

    let benchmark_name = "0-simple00-0.5.0";

    let insert = client.prepare("INSERT INTO job (name) VALUES ($1)")
                        .expect("Could not prepare statement.");

    client.execute(&insert, &[&benchmark_name]);

     for row in client.query("SELECT id,name FROM job", &[])? {
         let id: i32 = row.get(0);
         let name: &str = row.get(1);

         println!("found job: {} {}", id, name);
     }
}
