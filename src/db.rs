use config::ConfigParser;
use errors::AppError;
use ini::Ini;
use postgres::{Connection, TlsMode};
use postgres::params::{Host, ConnectParams};
use rust_decimal::Decimal;


const SELECT_ACCOUNT_SQL : &str =    "SELECT id FROM accounts WHERE name=$1";
const SELECT_ITEM_SQL : &str =       "SELECT id FROM items WHERE name=$1";
const SELECT_LAST_PRICE_SQL : &str = "SELECT price FROM items_prices \
                                      WHERE item_id=$1 AND account_id=$2 \
                                      ORDER BY effective_date DESC LIMIT 1";
const INSERT_ITEM_PRICE_SQL : &str = "INSERT INTO items_prices(item_id, account_id, price) \
                                      VALUES ($1, $2, $3)";

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn from_config(conf: &Ini) -> Result<Db, AppError> {
        let parser = ConfigParser::new(conf).section("database");
        let user = parser.get("user")?;
        let params = ConnectParams::builder()
            .user(user, parser.get_or_none("password"))
            .database(parser.get_or("database", "management"))
            .build(
                parser.get_or_none("host")
                      .map_or(
                           Host::Tcp("localhost".to_string()),
                           |host| Host::Tcp(host.to_string())
                      )
            );

        Ok(Db{
            conn: Connection::connect(params, TlsMode::None)?
        })
    }

    pub fn get_account_id(&self, account_name: &str) -> Result<i32, AppError> {
        let rows = self.conn.query(SELECT_ACCOUNT_SQL, &[&account_name])?;

        if rows.is_empty() {
            Err(AppError::Db("account not found".to_string()))
        } else {
            Ok(rows.get(0).get("id"))
        }
    }

    pub fn get_item_id(&self, item_name: &str) -> Result<i32, AppError> {
        let rows = self.conn.query(SELECT_ITEM_SQL, &[&item_name])?;

        if rows.is_empty() {
            Err(AppError::Db("item not found".to_string()))
        } else {
            Ok(rows.get(0).get("id"))
        }
    }

    pub fn get_last_price(&self, item_id: &i32, account_id: &i32) -> Result<Option<Decimal>, AppError> {
        let rows = self.conn.query(SELECT_LAST_PRICE_SQL, &[item_id, account_id])?;
        Ok(if rows.is_empty() { None } else { Some(rows.get(0).get("price")) })
    }

    pub fn insert_price(&self, item_id: &i32, account_id: &i32, price: &Decimal) -> Result<(), AppError> {
        self.conn.execute(INSERT_ITEM_PRICE_SQL, &[item_id, account_id, price])?;
        Ok(())
    }
}
