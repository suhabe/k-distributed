CREATE TABLE job (
   id serial primary key,
   name text,
   request_dt timestamp with time zone,
   s3_bucket text,
   s3_key text,
   timeout_sec int,
   processing_dt timestamp with time zone,
   result_dt timestamp with time zone,
   result_url text,
   completed_dt timestamp with time zone
);