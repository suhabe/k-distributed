extern crate serde;
extern crate difference;

use serde::Serialize;
use chrono::{Local,Utc};
use crate::job::{Job};

#[derive(Serialize, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Row {
    pub id: i32,
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
    pub result:String,
    pub result_color:String,
    //pub spec_url:String
}

#[derive(Serialize, Debug)]
pub struct TestCaseDetail {
    pub name:String,
    pub rows:Vec<Row>,
    pub diff:String

}

#[derive(Serialize, Debug)]
pub struct TestCaseSummary {
    pub name:String,
    pub passing:String,
    pub passing_color:String
}

#[derive(Serialize, Debug)]
pub struct Results {
    pub correct: TestCaseDetail,
    pub summary: Vec<TestCaseSummary>,
    pub details: Vec<TestCaseDetail>
}

fn to_local_str(x: chrono::DateTime<Utc>) -> String {
    x.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn e() -> String {
    String::from("")
}

pub fn row(j: &Job) -> Row {
    let result;
    let result_color;
    if j.completed_dt.is_some() {
        if let Some(true) = j.proved {
            result = String::from("proved true");
            result_color = "#98FB98";
        } else if let Some(false) = j.proved {
            result = String::from("proved false");
            result_color = "#FFCCCB";
        } else {
            if j.timed_out.is_some() && j.timed_out.unwrap() {
                result = String::from("timeout");
                result_color = "#FFCCCB";
            } else {
                result = String::from("error");
                result_color = "#FFCCCB";
            }
        }
    } else {
        result = String::from("incomplete");
        result_color = "#FFD300";
    }

    Row {
        id: j.id,
        completed_dt: j.completed_dt.map_or(e(), |x| to_local_str(x)),
        request_dt: j.request_dt.map_or(e(), |x| to_local_str(x)),
        processed_dt: j.processing_dt.map_or(e(), |x| to_local_str(x)),
        benchmark_name: j.benchmark_name.to_owned(),
        processing_secs: j.get_processing_secs().map_or(e(), |x| x.to_string()),
        processing_mins: j.get_processing_secs().map_or(e(), |x| format!("{:.1}", x as f64/60.)),
        spec_name: j.spec_name.to_owned(),
        status_code: match j.status_code {
            Some(c) => c.to_string(),
            None => e()
        },
        out_url: j.output_log_s3_url().unwrap_or(e()),
        err_url: j.error_log_s3_url().unwrap_or(e()),
        result,
        result_color: String::from(result_color),
        // spec_url: j.spec
    }
}
