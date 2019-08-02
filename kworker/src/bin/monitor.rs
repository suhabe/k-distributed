extern crate kworker;
extern crate env_logger;
extern crate handlebars;
extern crate serde;

use handlebars::Handlebars;
use std::error::Error;
use std::fs::{File,create_dir_all, remove_dir_all};
use std::env;
use kworker::job::get_jobs;
use kworker::db::exec;
use serde::Serialize;
use chrono::{Local,Utc};

#[derive(Serialize, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Row {
    pub completed_dt:String,
    pub request_dt:String,
    pub processed_dt:String,
    pub processing_secs:String,
    pub benchmark_name:String,
    pub spec_name:String,
    pub status_code:String,
    pub out_url:String,
    pub err_url:String,
    pub proved:String,
    pub proved_color:String
}

fn to_local_str(x: chrono::DateTime<Utc>) -> String {
    x.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string()
}

fn e() -> String {
    String::from("")
}

fn main() -> Result<(), Box<dyn Error>>  {
    dotenv::dotenv().ok();
    env_logger::init();

    let handlebars = Handlebars::new();

    let mut jobs = exec(|tx| get_jobs(tx,None));

    jobs.sort_by_key(|j| j.request_dt);

    let mut rows = Vec::new();
    for j in jobs {
        rows.push(Row {
            completed_dt: j.completed_dt.map_or(e(), |x| to_local_str(x)),
            request_dt: j.request_dt.map_or(e(), |x| to_local_str(x)),
            processed_dt: j.processing_dt.map_or(e(), |x| to_local_str(x)),
            benchmark_name: j.benchmark_name.to_owned(),
            processing_secs: j.get_processing_secs().map_or(e(), |x| x.to_string()),
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
            proved: j.proved.map_or(e(), |x| x.to_string()),
            proved_color: j.proved.map_or(String::from("#ffffff"), |x| match x {
                true => String::from("#98FB98"),
                false => String::from("#FFCCCB")
            })
        })
    }

    let kdist = env::var("KDIST_HOME").expect("KDIST_HOME not set");
    let template = format!("{}/kworker/ui/templates/monitor.hbs", kdist);
    let gendir = format!("{}/kworker/generated", kdist);
    let results_file = format!("{}/results.html", gendir);

    remove_dir_all(&gendir).is_ok();
    create_dir_all(&gendir).expect("Could not create generated directory");

    let mut source_template = File::open(template).unwrap();
    let mut output_file = File::create(&results_file).unwrap();
    handlebars.render_template_source_to_write(&mut source_template, &rows, &mut output_file).unwrap();
    println!("Generated results file {}", &results_file);
    Ok(())
}