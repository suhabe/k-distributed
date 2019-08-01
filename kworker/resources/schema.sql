CREATE TABLE job (
   id serial primary key,
   benchmark_name text not null,
   spec_name text not null,
   kprove text not null,
   semantics text not null,
   request_dt timestamp with time zone  not null,
   s3_bucket text not null,
   s3_key text not null,
   spec_filename text not null,
   timeout_sec int null,
   processing_dt timestamp with time zone,
   output_log_s3_key text,
   error_log_s3_key text,
   status_code int,
   timed_out boolean,
   proved boolean,
   completed_dt timestamp with time zone
);