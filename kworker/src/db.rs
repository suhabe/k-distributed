extern crate postgres;
extern crate postgres_native_tls;

use std::fs;
use std::env;

use native_tls::{Certificate, TlsConnector};
use postgres_native_tls::MakeTlsConnector;
use postgres::{Client, Transaction};

pub fn exec<F,T>(task: F) -> T where F: FnOnce(&mut Transaction) -> T {
    let hostname = &env::var("APP_DB_HOST").expect("APP_DB_HOST not set");
    let port = env::var("APP_DB_PORT").expect("APP_DB_PORT not set").parse::<u16>().expect("APP_DB_PORT not set");
    let username = &env::var("APP_DB_USER").expect("APP_DB_USER not set");
    let password = &env::var("APP_DB_PASS").expect("APP_DB_PASS not set");
    let rdscacert = &env::var("APP_RDS_CA_BUNDLE_PEM").expect("APP_RDS_CA_BUNDLE_PEM not set");

    let cert = fs::read(rdscacert).expect("Cannot find pem file used to AWS RDS");
    let cert = Certificate::from_pem(&cert).expect("Cannot parse pem file");
    let connector = TlsConnector::builder()
        .add_root_certificate(cert)
        .build()
        .expect("Cannot create connector");
    let connector = MakeTlsConnector::new(connector);

    let mut conn = Client::configure()
        .host(hostname)
        .user(username)
        .password(password)
        .port(port)
        .connect(connector)
        .expect("Could not connect to db.");

    let mut trans = conn.transaction().expect("Could not create transaction");

    let r = task(&mut trans);

    trans.commit().expect("Could not commit transaction");

    return r;
}