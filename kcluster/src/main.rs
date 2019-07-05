extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_rds;
extern crate postgres;
extern crate uuid;
extern crate postgres_native_tls;

use rusoto_core::Region;
use rusoto_rds::{RdsClient, Rds};
use rusoto_s3::{S3Client, S3, PutObjectRequest};
use uuid::Uuid;
use std::fs;
use std::env;

use native_tls::{Certificate, TlsConnector};
use postgres_native_tls::MakeTlsConnector;
use postgres::{Client, NoTls};

fn main() {
    connect_db();
}

/*
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
}*/

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
    let hostname = &env::var("APP_DB_HOST").expect("APP_DB_HOST not set");
    let port = env::var("APP_DB_PORT").expect("APP_DB_PORT not set").parse::<u16>().expect("APP_DB_PORT not set");
    let username = &env::var("APP_DB_USER").expect("APP_DB_USER not set");
    let password = &env::var("APP_DB_PASS").expect("APP_DB_PASS not set");
    let rdscacert = &env::var("APP_RDS_CA_BUNDLE_PEM").expect("APP_RDS_CA_BUNDLE_PEM not set");

    let cert = fs::read(rdscacert).expect("Cannot find pem file");
    let cert = Certificate::from_pem(&cert).expect("Cannot parse pem file");
    let connector = TlsConnector::builder()
        .add_root_certificate(cert)
        .build()
        .expect("Cannot create connector");
    let connector = MakeTlsConnector::new(connector);

    let mut client = Client::configure()
            .host(hostname)
            .user(username)
            .password(password)
            .port(port)
            .connect(connector)
            .expect("Could not connect to db.");

    let benchmark_name = "0-simple00-0.5.0";

    let insert = client.prepare("INSERT INTO job (name) VALUES ($1)")
                        .expect("Could not prepare statement.");

    client.execute(&insert, &[&benchmark_name]);

     for row in client.query("SELECT id,name FROM job", &[]).unwrap() {
         let id: i32 = row.get(0);
         let name: &str = row.get(1);

         println!("found job: {} {}", id, name);
     }
}
