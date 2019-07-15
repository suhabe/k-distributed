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
use postgres::{Client, NoTls, Transaction};

fn main() {
    exec(|trans| {
        for row in trans.query("SELECT id,name FROM job", &[]).unwrap() {
            let id: i32 = row.get(0);
            let name: &str = row.get(1);

            println!("found job: {} {}", id, name);
        }
    });
}

fn exec(task: fn(&mut Transaction)) {
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

    let mut conn = Client::configure()
        .host(hostname)
        .user(username)
        .password(password)
        .port(port)
        .connect(connector)
        .expect("Could not connect to db.");

    let mut trans = conn.transaction().unwrap();

    task(&mut trans);
}

