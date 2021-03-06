#![feature(proc_macro_hygiene, decl_macro, never_type)]

#[macro_use]
extern crate rocket;

mod api;
mod auth;
mod notifications;
mod pages;
mod state;

use rocket::fairing::AdHoc;
use rocket::http::uri::Uri;
use rocket::{Request, Rocket};
use rocket_contrib::database;
use rocket_contrib::databases::diesel;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;

use devand_mailer::Client as Mailer;
use serde::Serialize;

#[database("pg_devand")]
struct PgDevandConn(diesel::PgConnection);

pub struct StaticDir(pub String);

fn static_files(rocket: Rocket) -> Result<Rocket, Rocket> {
    const DEFAULT_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/static");

    let dir = rocket
        .config()
        .get_str("static_dir")
        .unwrap_or(DEFAULT_DIR)
        .to_string();

    let static_files = StaticFiles::from(dir.clone());

    let rocket = rocket.manage(StaticDir(dir)).mount("/static", static_files);

    Ok(rocket)
}

fn run_db_migrations(rocket: Rocket) -> Result<Rocket, Rocket> {
    let conn = PgDevandConn::get_one(&rocket).expect("database connection");
    match devand_db::run_migrations(&*conn) {
        Ok(()) => Ok(rocket),
        Err(_) => Err(rocket),
    }
}

fn init_wsmc(rocket: Rocket) -> Result<Rocket, Rocket> {
    let conn = PgDevandConn::get_one(&rocket).expect("database connection");
    let conn = conn.0;
    let wsmc = rocket.state::<WeekScheduleMatrix>();
    if let Some(wsmc) = wsmc {
        let mut wsmc = wsmc.0.write().unwrap();
        wsmc.init(&*conn);
    }
    Ok(rocket)
}

fn create_mailer() -> Mailer {
    let conf = devand_mailer::ClientConf {
        url: std::env::var("DEVAND_MAILER_SERVER_URL").unwrap(),
    };

    Mailer::new(conf)
}

#[derive(Default)]
struct CodeNowUsers(pub std::sync::RwLock<state::CodeNowUsersMap>);

#[derive(Default)]
struct WeekScheduleMatrix(pub std::sync::RwLock<state::WeekScheduleMatrixCache>);

#[catch(401)]
fn unauthorized(req: &Request) -> Template {
    #[derive(Serialize)]
    struct Context {
        title: &'static str,
        uri: String,
    }

    let uri = req.uri();
    let uri = Uri::percent_encode(&uri.to_string()).to_string();

    let context = Context {
        title: "Unauthorized",
        uri,
    };

    Template::render("unauthorized", &context)
}

#[catch(404)]
fn not_found() -> Template {
    #[derive(Serialize)]
    struct Context {
        title: &'static str,
        message: &'static str,
    }

    let context = Context {
        title: "Unauthorized",
        message: "The requested resource could not be found.",
    };

    Template::render("error", &context)
}

fn main() {
    env_logger::init();
    dotenv::dotenv().ok();

    let secret = std::env::var("DEVAND_SECRET").unwrap();
    let secret = secret.as_bytes();

    rocket::ignite()
        .manage(create_mailer())
        .manage(CodeNowUsers::default())
        .manage(WeekScheduleMatrix::default())
        .manage(devand_crypto::Decoder::new_from_secret(secret))
        .manage(devand_crypto::Encoder::new_from_secret(secret))
        .attach(Template::fairing())
        .attach(PgDevandConn::fairing())
        .attach(AdHoc::on_attach("Database Migrations", run_db_migrations))
        .attach(AdHoc::on_attach("Static files", static_files))
        .attach(AdHoc::on_attach("WeekScheduleMatrixCache", init_wsmc))
        .mount("/", pages::routes())
        .mount("/api", api::routes())
        .register(catchers![not_found, unauthorized])
        .launch();
}
