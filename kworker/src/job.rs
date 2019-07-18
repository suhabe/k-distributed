extern crate postgres;
extern crate chrono;

use std::fs;
use std::env;
use postgres::Transaction;

use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Job {
    id: i32,
    name: String,
    request_dt: Option<chrono::DateTime<Utc>>,
    s3_bucket: Option<String>,
    s3_key: Option<String>,
    timeout_sec: Option<i32>,
    processing_dt: Option<chrono::DateTime<Utc>>,
    result_dt: Option<chrono::DateTime<Utc>>,
    result_url: Option<String>,
    completed_dt: Option<chrono::DateTime<Utc>>
}

pub fn list_jobs(trans: &mut Transaction) {
    for job in get_jobs(trans) {
        println!("{:?}", job);
    }
}

//benchmark_name, bucket_name, benchmark_key, 1800
pub fn new_job(trans: &mut Transaction, benchmark_name: &String, bucket_name: &String, benchmark_key: &String, timeout_sec: i32) {
    let request_dt = Utc::now();
    trans.execute("INSERT INTO job (name,request_dt,s3_bucket,s3_key,timeout_sec) VALUES ($1,$2,$3,$4,$5)",
                  &[benchmark_name, &request_dt, bucket_name, benchmark_key, &timeout_sec]).unwrap();
}

pub fn reset_job(trans: &mut Transaction) -> i32  {
    trans.execute("UPDATE job SET processing_dt = null", &[]).unwrap();
    0
}

pub fn pop_job(trans: &mut Transaction) -> Option<i32> {
    let mut jobs = get_jobs(trans);
    let pop: Option<Job> = jobs.into_iter().find(|j| j.processing_dt.is_none());
    match pop {
        Some(job) => {
            let now = Utc::now();
            trans.execute("UPDATE job SET processing_dt = $1 WHERE id = $2", &[&now, &job.id]).unwrap();
            Some(job.id)
        }
        None => None
    }
}

pub fn get_jobs(trans: &mut Transaction) -> Vec<Job> {
    let mut jobs = Vec::new();
    let fields = vec![
        "id",
        "name",
        "request_dt",
        "s3_bucket",
        "s3_key",
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
        let s3_bucket: Option<String> = row.get("s3_bucket");
        let s3_key: Option<String> = row.get("s3_key");
        let timeout_sec: Option<i32> = row.get("timeout_sec");
        let processing_dt: Option<DateTime<Utc>> = row.get("processing_dt");
        let result_dt: Option<DateTime<Utc>> = row.get("result_dt");
        let result_url: Option<String> = row.get("result_url");
        let completed_dt: Option<DateTime<Utc>> = row.get("completed_dt");

        let job = Job {
            id,
            name,
            request_dt,
            s3_bucket,
            s3_key,
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
