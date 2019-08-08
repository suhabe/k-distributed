extern crate kworker;
extern crate env_logger;
extern crate handlebars;
extern crate serde;

use handlebars::Handlebars;
use std::error::Error;
use std::fs::{File,create_dir_all, remove_dir_all};
use std::env;
use kworker::job::*;
use kworker::db::exec;
use serde::Serialize;
use chrono::{Local,Utc};
use itertools::Itertools;


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
    pub rows:Vec<Row>
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
        proved: proved.map_or(e(), |x| x.to_string()),
        proved_color: proved.map_or(String::from("#ffffff"), |x| match x {
            true => String::from("#98FB98"),
            false => String::from("#FFCCCB")
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    env_logger::init();

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


    let mut jobs = exec(|tx| get_jobs(tx, None));
    jobs.sort_by(|j1,j2| j1.benchmark_name.cmp(&j2.benchmark_name)
        .then(j1.request_dt.cmp(&j2.request_dt)));

    let cutoff = chrono::DateTime::parse_from_rfc3339("2019-08-04T00:00:00+00:00").unwrap().with_timezone(&Utc);
    let prefix = "TEST-";

    let grouped_jobs = jobs.iter()
        .filter(|j| j.request_dt.unwrap().ge(&cutoff))
        .filter(|j| j.benchmark_name.starts_with(prefix))
        .group_by(|j| j.benchmark_name.to_owned());

    let mut pgms = Vec::new();
    for (key,group) in &grouped_jobs {
        pgms.push(Pgm {
            name: key,
            rows: group.into_iter().map(|j| row(&j)).collect()
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