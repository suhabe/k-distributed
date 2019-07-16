extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_rds;
extern crate postgres;
extern crate uuid;
extern crate postgres_native_tls;
extern crate chrono;

use rusoto_core::Region;
use rusoto_rds::{RdsClient, Rds};
use rusoto_s3::{S3Client, S3, PutObjectRequest};
use uuid::Uuid;
use std::fs;
use std::env;

use native_tls::{Certificate, TlsConnector};
use postgres_native_tls::MakeTlsConnector;
use postgres::{Client, NoTls, Transaction};
use chrono::{DateTime, Utc};

fn main() {
    exec(|trans| {
        for job in get_jobs(trans) {
            if job.processing_dt.is_some() {

            }
            println!("{}", job.name);
        }
    });
}

fn pop_job() {
    exec(|trans| {
        get_jobs(trans).iter().find(|&&job| job.processing_dt.is_none());
    });
}

struct Job {
    id: i32,
    name: String,
    request_dt: Option<chrono::DateTime<Utc>>,
    request_url: Option<String>,
    timeout_sec: Option<i32>,
    processing_dt: Option<chrono::DateTime<Utc>>,
    result_dt: Option<chrono::DateTime<Utc>>,
    result_url: Option<String>,
    completed_dt: Option<chrono::DateTime<Utc>>
}

fn get_jobs(trans: &mut Transaction) -> Vec<Job> {
    let mut jobs = Vec::new();
    let fields = vec![
        "id",
        "name",
        "request_dt",
        "request_url",
        "timeout_sec",
        "processing_dt",
        "result_dt",
        "result_url",
        "completed_dt"
    ];
    let select_fields = fields.join(",");
    let query = format!("SELECT {} FROM job", select_fields);

    for row in trans.query(query.as_str(), &[]).unwrap() {

        let id: i32 = row.get("id");
        let name: String = row.get("name");
        let request_dt: Option<DateTime<Utc>> = row.get("request_dt");
        let request_url: Option<String> = row.get("request_url");
        let timeout_sec: Option<i32> = row.get("timeout_sec");
        let processing_dt: Option<DateTime<Utc>> = row.get("processing_dt");
        let result_dt: Option<DateTime<Utc>> = row.get("result_dt");
        let result_url: Option<String> = row.get("result_url");
        let completed_dt: Option<DateTime<Utc>> = row.get("completed_dt");

        let job = Job {
            id,
            name,
            request_dt,
            request_url,
            timeout_sec,
            processing_dt,
            result_dt,
            result_url,
            completed_dt
        };

        jobs.push(job);

    }

    jobs
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

