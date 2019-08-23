extern crate serde;
extern crate kworker;
extern crate fs_extra;

use actix_web::{middleware::Logger, web, App, HttpServer};
use serde::{Serialize,Deserialize};
use kworker::view::{Row,e};
use actix_files as fs;
use uuid::{Uuid};
use fs_extra::dir::copy;
use fs_extra::dir::CopyOptions;
use std::io::prelude::*;


#[derive(Deserialize)]
struct ProveRequest {
    program:  String,
    spec:  String,
}

#[derive(Serialize)]
struct ProveResponse {
    row: Row
}

fn prove(pr: web::Json<ProveRequest>) -> Result<web::Json<ProveResponse>,actix_web::Error> {
    println!("Program: {}", pr.program);
    println!("Spec: {}", pr.spec);

    let uuid = Uuid::new_v4();
    let tmpldir = "/home/sbugrara/k-distributed/kdev/sandbox";
    let tmpdir = format!("/tmp/kdev-{}", &uuid);
    let sandboxdir = format!("/tmp/kdev-{}/sandbox", &uuid);
    println!("Creating sandbox: {} from {}", tmpdir, tmpldir);
    std::fs::create_dir_all(&tmpdir).expect("Cannot create tmp");
    copy(&tmpldir, &tmpdir, &CopyOptions::new()).expect("Cannot create directory");

    let mut program_file = std::fs::File::create(format!("{}/program/program.sol",&sandboxdir)).expect("Cannot create solidity file.");
    program_file.write_all(pr.program.as_bytes()).expect("Could not write program");

    let mut spec_file = std::fs::File::create(format!("{}/program/spec.ini",&sandboxdir)).expect("Cannot create spec file.");
    spec_file.write_all(pr.spec.as_bytes()).expect("Could not write spec");

    

    Ok(web::Json(ProveResponse {
        row: Row {
            completed_dt: e(),
            request_dt: e(),
            processed_dt: e(),
            processing_secs: e(),
            processing_mins: e(),
            benchmark_name: e(),
            spec_name: e(),
            status_code: String::from("0"),
            out_url: String::from("https://stdout"),
            err_url: String::from("https://stderr"),
            result: String::from("proved true"),
            result_color: String::from("#ff0000"),
        }
    }))
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
            .service(fs::Files::new("/", "/home/sbugrara/k-distributed/kdev/client/build").index_file("index.html"))
        })
        .bind("127.0.0.1:8080")
        .expect("Can not bind to port 8000")
        .run()
        .unwrap();
}