extern crate curl;
extern crate ini;
#[macro_use] extern crate log;
extern crate simplelog;
extern crate postgres;
extern crate regex;
extern crate rust_decimal;

use config::ConfigParser;
use db::Db;
use errors::AppError;
use http::HttpProcessor;
use ini::Ini;
use simplelog::{Config, LevelFilter, SimpleLogger};

use std::env;
use std::str::FromStr;
use std::process::exit;

mod db;
mod http;
mod config;
mod errors;


fn run(filename: &str) -> Result<(), AppError> {
    let config = load_ini(filename)?;
    if let Some(level_str) = ConfigParser::new(&config).section("log").get_or_none("level") {
        match LevelFilter::from_str(level_str) {
            Ok(level) => { SimpleLogger::init(level, Config::default()); },
            Err(_)    => { return Err(AppError::Config(format!("wrong log level '{}'", level_str))); },
        }
    } else {
        SimpleLogger::init(LevelFilter::Info, Config::default());
    }

    let parser = ConfigParser::new(&config).section("processing");
    let item_name = parser.get("item")?;
    let account_name = parser.get("account")?;
    debug!("item name='{}'account name='{}'", item_name, account_name);

    let db = Db::from_config(&config)?;
    let item_id = db.get_item_id(item_name)?;
    let account_id = db.get_account_id(account_name)?;
    let last_price = db.get_last_price(&item_id, &account_id)?;
    debug!("last price='{:?}'", last_price);

    let http_processor = HttpProcessor::from_config(&config)?;
    let current_price = http_processor.fetch_price()?;

    match current_price {
        Some(cp) => {
            info!("current price: {}", cp);
            if match last_price {
                Some(lp) => {
                    if cp < lp {
                        info!("current price: {} last stored price: {} => price decreases", cp, lp);
                    } else if cp > lp {
                        info!("current price: {} last stored price: {} => price increases", cp, lp);
                    }
                    cp != lp
                },
                None => true
            } {
                db.insert_price(&item_id, &account_id, &cp)?;
            }
        },
        None => {
            warn!("no price found");
        }
    }
    Ok(())
}

fn load_ini(filename: &str) -> Result<Ini, AppError> {
    Ok(Ini::load_from_file(filename)?)
}

fn main() {
    match env::args().nth(1) {
        Some(filename) => {
            run(&filename).unwrap_or_else(|e| {
                error!("{}", e);
                exit(1);
            });
        },
        None => {
            println!("USAGE: price_sniffer <config.ini>");
            exit(1);
        }
    }
}
