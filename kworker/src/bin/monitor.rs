extern crate kworker;
extern crate env_logger;
extern crate handlebars;
extern crate serde;
extern crate difference;


use handlebars::Handlebars;
use std::error::Error;
use std::fs::*;
use std::env;
use kworker::job::*;
use kworker::db::exec;
use serde::Serialize;
use chrono::{Local,Utc};
use itertools::Itertools;
use difference::Changeset;
use kworker::s3::s3_download_dir;
use rusoto_s3::{S3Client, S3, PutObjectRequest, ListObjectsRequest, GetObjectRequest};
use rusoto_core::Region;
use std::process::Command;

#[derive(Serialize, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Row {
    pub completed_dt:String,
    pub request_dt:String,
    pub processed_dt:String,
    pub processing_secs:String,
    pub processing_mins:String,
    pub benchmark_name:String,
    pub spec_name:String,
    pub status_code:String,
    pub out_url:String,
    pub err_url:String,
    pub proved:String,
    pub proved_color:String
}

#[derive(Serialize, Debug)]
struct Pgm {
    pub name:String,
    pub rows:Vec<Row>,
    pub diff:String

}

fn to_local_str(x: chrono::DateTime<Utc>) -> String {
    x.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string()
}

fn e() -> String {
    String::from("")
}

fn row(j: &Job) -> Row {
    let proved;
    if j.completed_dt.is_some() {
        proved = Some(j.proved.is_some() && j.proved.unwrap());
    } else {
        proved = None;
    }

    Row {
        completed_dt: j.completed_dt.map_or(e(), |x| to_local_str(x)),
        request_dt: j.request_dt.map_or(e(), |x| to_local_str(x)),
        processed_dt: j.processing_dt.map_or(e(), |x| to_local_str(x)),
        benchmark_name: j.benchmark_name.to_owned(),
        processing_secs: j.get_processing_secs().map_or(e(), |x| x.to_string()),
        processing_mins: j.get_processing_secs().map_or(e(), |x| format!("{:.1}", x as f64/60.)),
        spec_name: j.spec_name.to_owned(),
        status_code: match j.status_code {
            Some(c) => c.to_string(),
            None => match j.timed_out {
                Some(true) => String::from("Timed out"),
                _ => e()
            }
        },
        out_url: j.output_log_s3_url().unwrap_or(e()),
        err_url: j.error_log_s3_url().unwrap_or(e()),
        proved: proved.map_or(e(), |x| if x { String::from("yes") } else { String::from("no") }),
        proved_color: proved.map_or(String::from("#ffffff"), |x| match x {
            true => String::from("#98FB98"),
            false => String::from("#FFCCCB")
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    env_logger::init();

    gen_tests()?;

    Ok(())
}

fn gen_monitor() -> Result<(), Box<dyn Error>> {
    let mut jobs = exec(|tx| get_jobs(tx, None));
    jobs.sort_by_key(|j| j.request_dt);
    let mut rows = Vec::new();
    for j in jobs {
        rows.push(row(&j));
    }

    let kdist = env::var("KDIST_HOME").expect("KDIST_HOME not set");
    let gendir = format!("{}/kworker/generated", kdist);
    remove_dir_all(&gendir).is_ok();
    create_dir_all(&gendir).expect("Could not create generated directory");

    let monitor_template = format!("{}/kworker/ui/templates/monitor.hbs", kdist);
    let monitor_file = format!("{}/monitor.html", gendir);
    generate(monitor_template, monitor_file, &rows);

    let report_template = format!("{}/kworker/ui/templates/report.hbs", kdist);
    let report_file = format!("{}/report.html", gendir);
    generate(report_template, report_file, &rows);

    Ok(())
}

fn gen_tests() -> Result<(), Box<dyn Error>> {
    let client = S3Client::new(Region::UsEast2);

    let mut jobs = exec(|tx| get_jobs(tx, None));
    jobs.sort_by(|j1,j2| j1.benchmark_name.cmp(&j2.benchmark_name)
        .then(j1.request_dt.cmp(&j2.request_dt)));

    let cutoff = chrono::DateTime::parse_from_rfc3339("2019-08-04T00:00:00+00:00").unwrap().with_timezone(&Utc);
    let prefix = "TEST-";
    let diff_source_file = "/home/sbugrara/kevm-verify-benchmarks/multisig13/multisig13.sol";
    let kdist = env::var("KDIST_HOME").expect("KDIST_HOME not set");
    let gendir = format!("{}/kworker/generated", kdist);
    let tmpdir = String::from("/tmp/k-distributed");
    let resourcesdir = "/home/sbugrara/k-distributed/kworker/resources";

    let grouped_jobs = jobs.iter()
        .filter(|j| j.request_dt.unwrap().ge(&cutoff))
        .filter(|j| j.benchmark_name.starts_with(prefix))
        .group_by(|j| j.benchmark_name.to_owned());

    remove_dir_all(&tmpdir)?;
    create_dir(&tmpdir)?;

    let mut pgms = Vec::new();
    for (key,group) in &grouped_jobs {
        let jobs: Vec<&Job> = group.into_iter().collect();
        let rows = jobs.iter().map(|j| row(&j)).collect();

        let job = jobs[0];
        let s3_bucket = job.s3_bucket.as_ref().unwrap();
        let s3_key = job.s3_key.as_ref().unwrap();

        let down_dir = s3_download_dir(&client, &s3_bucket, &s3_key, Some(|k:&String| k.ends_with(".sol")), &tmpdir);
        let sol_file = read_dir(&down_dir).unwrap().next().unwrap().unwrap().path();
        println!("sol_file: {:?}", &sol_file);
        println!("diff_source_file: {:?}", diff_source_file);

        let diff_file_path = format!("{}/{}", down_dir.to_str().unwrap(), "diff.txt");
        let diffhtml_file_path = format!("{}/{}", down_dir.to_str().unwrap(), "diff.html");
        let tmpl_file_path = format!("{}/{}", resourcesdir, "tmpl.html");
        println!("diff_file_path: {:?}", diff_file_path);

        let args = ["-U", "100000", "-u",
            diff_source_file,
            sol_file.to_str().unwrap()];

        println!("args: {:?}", args);
        let output = Command::new("diff")
            .args(&args)
            .output()
            .expect("failed to execute process");

        std::fs::write(&diff_file_path, output.stdout).expect("Could not write file");

        let args = ["-i", "file",
            "--su", "hidden",
            "-F", &diffhtml_file_path,
            "--hwt", &tmpl_file_path,
            "--", &diff_file_path];

        println!("diff2html {}", args.join(" "));

        let output = Command::new("diff2html")
            .args(&args)
            .output()
            .expect("failed to execute process");

        println!("diffhtml_file_path: {:?}", diffhtml_file_path);

        let diffhtml = read_to_string(diffhtml_file_path).unwrap();

        pgms.push(Pgm {
            name: key,
            rows: rows,
            diff: diffhtml
        });

    }

    let tests_template = format!("{}/kworker/ui/templates/tests.hbs", kdist);
    let tests_file = format!("{}/tests.html", gendir);
    generate(tests_template, tests_file, &pgms);

    Ok(())
}

fn generate<T>(template_path: String, output_path: String, rows: &Vec<T>) where T: Serialize {
    let handlebars = Handlebars::new();
    let mut template_file = File::open(template_path).unwrap();
    let mut output_file = File::create(&output_path).unwrap();
    handlebars.render_template_source_to_write(&mut template_file, &rows, &mut output_file).unwrap();
    println!("Generated file {}", &output_path);

}