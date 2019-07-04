CREATE TABLE job (
   id serial primary key,
   name text,
   request_dt timestamp with time zone,
   request_url text,
   timeout_sec int,
   processing_dt timestamp with time zone,
   result_dt timestamp with time zone,
   result_url text,
   completed_dt timestamp with time zone
);