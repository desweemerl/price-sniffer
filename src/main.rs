extern crate reqwest;
#[macro_use] extern crate log;
extern crate simplelog;
extern crate postgres;
extern crate regex;
extern crate rust_decimal;
extern crate yaml_rust;

use config::ConfigParser;
use db::Db;
use errors::AppError;
use http::HttpProcessor;
use simplelog::{Config, LevelFilter, SimpleLogger};

use std::env;
use std::str::FromStr;
use std::process::exit;

mod db;
mod http;
mod config;
mod errors;


fn run<'a>(filename: &'a str) -> Result<(), AppError> {
    let cfg = ConfigParser::new(&filename)?;
    let level_str = cfg.clone().section("log")
       .get_str_or("level", "info");

    println!("level: {}", level_str);

    let level = LevelFilter::from_str(&level_str)
       .map_err(|_| AppError::Config(format!("wrong log level")))?;

    SimpleLogger::init(level, Config::default()).unwrap();

    let cfg_processing = cfg.clone().section("processing");
    let item_name = cfg_processing.get_str("item")?;
    let account_name = cfg_processing.get_str("account")?;
    debug!("item name='{}' account name='{}'", item_name, account_name);

    let db = Db::from_config(&cfg)?;
    let item_id = db.get_item_id(&item_name)?;
    let account_id = db.get_account_id(&account_name)?;
    let last_price = db.get_last_price(&item_id, &account_id)?;
    if let Some(lp) = last_price {
        debug!("last price='{:?}'", lp);
    }

    let http_processor = HttpProcessor::from_config(&cfg)?;
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
                None     => true,
            } {
                db.insert_price(&item_id, &account_id, &cp)?;
            }
        },
        None     => warn!("no price found"),
    }
    Ok(())
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
            println!("USAGE: price_sniffer <config.yaml>");
            exit(1);
        }
    }
}
