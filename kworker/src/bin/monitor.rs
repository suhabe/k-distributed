extern crate kworker;
extern crate env_logger;
extern crate handlebars;
extern crate serde;

use serde::Serialize;
use handlebars::Handlebars;
use std::error::Error;
use std::fs::{File,create_dir_all, remove_dir_all};
use std::env;
use kworker::job::get_jobs;
use kworker::db::exec;

#[derive(Serialize)]
struct Person {
    name: String,
    age: i16,
}

fn main() -> Result<(), Box<dyn Error>>  {
    dotenv::dotenv().ok();
    env_logger::init();

    let handlebars = Handlebars::new();

    let jobs = exec(|tx| get_jobs(tx,None));

    let kdist = env::var("KDIST_HOME").expect("KDIST_HOME not set");
    let template = format!("{}/kworker/ui/templates/monitor.hbs", kdist);
    let gendir = format!("{}/kworker/generated", kdist);
    let results_file = format!("{}/results.html", gendir);

    remove_dir_all(&gendir).is_ok();
    create_dir_all(&gendir).expect("Could not create generated directory");

    let mut source_template = File::open(template).unwrap();
    let mut output_file = File::create(&results_file).unwrap();
    handlebars.render_template_source_to_write(&mut source_template, &jobs, &mut output_file).unwrap();
    println!("Generated results file {}", &results_file);
    Ok(())
}