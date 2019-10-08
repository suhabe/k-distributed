extern crate serde;
extern crate kworker;
extern crate fs_extra;

use actix_web::{middleware::Logger, web, App, HttpServer};
use serde::{Serialize,Deserialize};
use kworker::view::{Row};
use kworker::db::{exec};
use kworker::job::*;
use kworker::s3::s3_upload_dir;
use uuid::{Uuid};
use fs_extra::dir::copy;
use fs_extra::dir::CopyOptions;
use std::io::Write;

#[derive(Deserialize)]
pub struct ProveRequest {
    program:  String,
    spec:  String,
}

#[derive(Deserialize)]
pub struct ReloadRequest {
    job_ids:  Vec<i32>
}

#[derive(Serialize)]
pub struct ProveResponse {
    jobs: Vec<Row>
}

fn prove(pr: web::Json<ProveRequest>) -> Result<web::Json<ProveResponse>,actix_web::Error> {
    println!("Program: {}", pr.program);
    println!("Spec: {}", pr.spec);

    let uuid = Uuid::new_v4();
    let tmpldir = "/home/sbugrara/k-distributed/kdev/sandbox";
    let tmpdir = format!("/tmp/kdev-{}", &uuid);
    let sandboxdir = format!("/tmp/kdev-{}/sandbox", &uuid);
    let programdir = format!("{}/program",&sandboxdir);
    let generateddir = format!("{}/generated",&programdir);
    println!("Creating sandbox: {} from {}", tmpdir, tmpldir);
    std::fs::create_dir_all(&tmpdir).expect("Cannot create tmp");
    copy(&tmpldir, &tmpdir, &CopyOptions::new()).expect("Cannot create directory");

    let mut program_file = std::fs::File::create(format!("{}/program.sol",&programdir)).expect("Cannot create solidity file.");
    program_file.write_all(pr.program.as_bytes()).expect("Could not write program");

    let mut spec_file = std::fs::File::create(format!("{}/spec.ini",&programdir)).expect("Cannot create spec file.");
    spec_file.write_all(pr.spec.as_bytes()).expect("Could not write spec");

    let output = std::process::Command::new("make")
        .current_dir(&programdir)
        .output()
        .expect("failed to execute process");

    println!("status: {}", output.status);
    std::io::stdout().write_all(&output.stdout).unwrap();
    std::io::stderr().write_all(&output.stderr).unwrap();

    assert!(output.status.success());

    let mut job_ids = Vec::new();
    for entry in std::fs::read_dir(&generateddir).expect("Cannot read generated") {
        let entry = entry.expect("Cannot read entry");
        let path = entry.path();
        let spec_filename = String::from(std::path::PathBuf::from(path).file_name().unwrap().to_str().unwrap());
        if !spec_filename.ends_with("spec.k") {
            continue;
        }

        let dirpath = std::path::PathBuf::from(&programdir);
        let kprove = String::from("k");
        let semantics = String::from("evm-semantics");

        let dirpathbuf = std::fs::canonicalize(&dirpath).unwrap();
        let benchmark_dir = String::from(dirpathbuf.as_path().to_str().unwrap());
        let benchmark_name = String::from(dirpath.file_name().unwrap().to_str().unwrap());
        let benchmark_key = format!("{}-{}", benchmark_name, Uuid::new_v4());
        //let job_name = format!("{}:{}", benchmark_ name, spec_filename);

        let client = rusoto_s3::S3Client::new(rusoto_core::Region::UsEast2);
        let bucket_name = String::from("kjob");

        s3_upload_dir(&client, &bucket_name, &benchmark_key, &benchmark_dir);

        let id = exec(|trans| { new_job(trans, &benchmark_name, &spec_filename, &kprove, &semantics, &bucket_name, &benchmark_key, &spec_filename, 3600 * 3) });
        job_ids.push(id);
    }

    Ok(web::Json(get_response(&job_ids)))
}

pub fn reload(rr: web::Json<ReloadRequest>) -> Result<web::Json<ProveResponse>,actix_web::Error> {
    Ok(web::Json(get_response(&rr.job_ids)))
}

pub fn get_response(job_ids: &Vec<i32>) -> ProveResponse {
    let mut rows = Vec::new();

    for job_id in job_ids {
        let job = exec(|tx| get_job(tx, *job_id));
        rows.push(kworker::view::row(&job));
    }

    ProveResponse {
        jobs: rows
    }
}

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();
    std::env::set_var("RUST_LOG", "my_errors=debug,actix_web=info");
    std::env::set_var("RUST_BACKTRACE", "1");

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(web::resource("/prove").route(web::post().to(prove)))
            .service(web::resource("/reload").route(web::post().to(reload)))
            .service(actix_files::Files::new("/", "/home/sbugrara/k-distributed/kdev/client/build").index_file("index.html"))
        })
        .bind("127.0.0.1:8080")
        .expect("Can not bind to port 8000")
        .run()
        .unwrap();
}