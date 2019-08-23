extern crate postgres;
extern crate chrono;
extern crate serde;

use postgres::Transaction;
use serde::Serialize;
use chrono::{DateTime, Utc};
use crate::s3::{s3_url_opt};

#[derive(Debug,Serialize)]
pub struct Job {
    pub id: i32,
    pub benchmark_name: String,
    pub spec_name: String,
    pub kprove: String,
    pub semantics: String,
    pub request_dt: Option<chrono::DateTime<Utc>>,
    pub s3_bucket: Option<String>,
    pub s3_key: Option<String>,
    pub spec_filename: String,
    pub timeout_sec: Option<i32>,
    pub processing_dt: Option<chrono::DateTime<Utc>>,
    pub output_log_s3_key: Option<String>,
    pub error_log_s3_key: Option<String>,
    pub status_code: Option<i32>,
    pub completed_dt: Option<chrono::DateTime<Utc>>,
    pub timed_out: Option<bool>,
    pub proved: Option<bool>,
}

impl Job {
    pub fn output_log_s3_url(self: &Job) -> Option<String> {
        s3_url_opt(self.s3_bucket.to_owned(), self.output_log_s3_key.to_owned())
    }

    pub fn error_log_s3_url(self: &Job) -> Option<String> {
        s3_url_opt(self.s3_bucket.to_owned(), self.error_log_s3_key.to_owned())
    }

    pub fn spec_url(self: &Job) -> Option<String> {
        s3_url_opt(self.s3_bucket.to_owned(), Some(format!("{}/{}", "generated", self.spec_filename.to_owned())))
    }

    pub fn get_processing_secs(self: &Job) -> Option<i64> {
        Some(self.completed_dt?.signed_duration_since(self.processing_dt?).num_seconds())
    }
}

pub fn get_job(tx: &mut Transaction, job_id: i32) -> Job {
    let jobs = get_jobs(tx, Some(job_id));
    jobs.into_iter().next().unwrap()
}

pub fn list_jobs(tx: &mut Transaction) {
    let jobs = get_jobs(tx, None);

    if jobs.is_empty() {
        info!("Job queue is empty");
    }
    for job in jobs {
        info!("{:?}", job);
    }
}

//benchmark_name, bucket_name, benchmark_key, 1800
pub fn new_job(tx: &mut Transaction, benchmark_name: &String, spec_name: &String, kprove: &String, semantics: &String, bucket_name: &String, benchmark_key: &String, spec_filename: &String, timeout_sec: i32) -> i32 {
    let request_dt = Utc::now();
    let result = tx.query("INSERT INTO job (benchmark_name,spec_name,kprove,semantics,request_dt,s3_bucket,s3_key,spec_filename,timeout_sec) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) RETURNING id",
                  &[benchmark_name, spec_name, kprove, semantics, &request_dt, bucket_name, benchmark_key, spec_filename, &timeout_sec]);
    let id: i32 = result.unwrap().iter().next().unwrap().get(0);
    id
}

pub fn reset_jobs(tx: &mut Transaction) -> i32  {
    tx.execute("UPDATE job SET processing_dt = null", &[]).unwrap();
    0
}

pub fn delete_jobs(tx: &mut Transaction) -> i32  {
    tx.execute("DELETE from job", &[]).unwrap();
    0
}

pub fn complete_job(tx: &mut Transaction, id: i32, output_log_s3_key: &String, error_log_s3_key: &String, status_code: Option<i32>, timed_out: bool, proved: Option<bool>) -> i32  {
    let now = Utc::now();
    tx.execute("UPDATE job SET output_log_s3_key = $1, error_log_s3_key = $2, status_code = $3, completed_dt = $4, timed_out = $5, proved = $6 where id = $7",
               &[&output_log_s3_key, &error_log_s3_key, &status_code, &now, &timed_out, &proved, &id]).unwrap();
    0
}

pub fn pop_job(tx: &mut Transaction) -> Option<i32> {
    let jobs = get_jobs(tx, None);
    let pop: Option<Job> = jobs.into_iter().find(|j| j.processing_dt.is_none());
    match pop {
        Some(job) => {
            let now = Utc::now();
            tx.execute("UPDATE job SET processing_dt = $1 WHERE id = $2", &[&now, &job.id]).unwrap();
            Some(job.id)
        }
        None => None
    }
}

pub fn get_jobs(tx: &mut Transaction, job_id: Option<i32>) -> Vec<Job> {
    let mut jobs = Vec::new();

    let condition;
    match job_id {
        Some(id) => {
            condition = format!("where id={}",id);
        },
        None => {
            condition = String::from("");
        }
    }
    let query = format!("SELECT * FROM job {}", condition);

    for row in tx.query(query.as_str(), &[]).unwrap() {

        let id: i32 = row.get("id");
        let benchmark_name: String = row.get("benchmark_name");
        let spec_name: String = row.get("spec_name");
        let kprove: String = row.get("kprove");
        let semantics: String = row.get("semantics");
        let request_dt: Option<DateTime<Utc>> = row.get("request_dt");
        let s3_bucket: Option<String> = row.get("s3_bucket");
        let s3_key: Option<String> = row.get("s3_key");
        let spec_filename = row.get("spec_filename");
        let timeout_sec: Option<i32> = row.get("timeout_sec");
        let processing_dt: Option<DateTime<Utc>> = row.get("processing_dt");
        let output_log_s3_key: Option<String> = row.get("output_log_s3_key");
        let error_log_s3_key: Option<String> = row.get("error_log_s3_key");
        let status_code: Option<i32> = row.get("status_code");
        let completed_dt: Option<DateTime<Utc>> = row.get("completed_dt");
        let timed_out: Option<bool> = row.get("timed_out");
        let proved: Option<bool> = row.get("proved");

        let job = Job {
            id,
            benchmark_name,
            spec_name,
            kprove,
            semantics,
            request_dt,
            s3_bucket,
            s3_key,
            spec_filename,
            timeout_sec,
            processing_dt,
            output_log_s3_key,
            error_log_s3_key,
            status_code,
            completed_dt,
            timed_out,
            proved
        };

        jobs.push(job);

    }

    jobs
}
