use config::ConfigParser;
use curl::easy::Easy;
use ini::Ini;
use errors::AppError;
use rust_decimal::Decimal;
use regex::Regex;

use std::io::Read;
use std::str;
use std::str::FromStr;


enum HttpMethod {
    GET,
    POST,
}

impl FromStr for HttpMethod {
    type Err = AppError;

    fn from_str(method: &str) -> Result<HttpMethod, AppError> {
        match &method.to_uppercase() as &str {
            "GET"  => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            _      => Err(AppError::ParseError(format!("invalid http method: {}", method)))
        }
    }
}

pub struct HttpProcessor<'a> {
    url: &'a str,
    method: HttpMethod,
    payload: Option<&'a str>,
    re: Regex,
}

impl<'a> HttpProcessor<'a> {

    pub fn from_config<'b: 'a>(conf: &'b Ini) -> Result<Self, AppError> {
        let parser = ConfigParser::new(conf).section("processing");
        let url = parser.get("url")?;
        debug!("url='{}'", url);
        let regex_str = parser.get("regex")?;
        debug!("regex='{}'", regex_str);
        let re = regex::RegexBuilder::new(regex_str).multi_line(true).build()?;
        let method_str = parser.get_or("method", "GET");
        debug!("method='{}'", method_str);
        let method = HttpMethod::from_str(method_str)?;
        let payload = parser.get_or_none("payload");
        debug!("payload='{:?}'", payload);

        Ok(HttpProcessor{
            url,
            method,
            payload,
            re,
        })
    }

    pub fn fetch_price(&self) -> Result<Option<Decimal>, AppError> {
        let response = match self.method {
            HttpMethod::POST => self.post(),
            _                => panic!("unsupported method"), /* TODO: support for other http methods */
        }?;

        match self.re.captures(str::from_utf8(&response).unwrap()).unwrap().get(1) {
            Some(v) => {
                match Decimal::from_str(v.as_str()) {
                    Ok(p) => Ok(Some(p)),
                    _     => Ok(None),
                }
            },
            _ => Ok(None)
        }
    }

    fn post(&self) -> Result<Vec<u8>, AppError> {
        let mut easy = Easy::new();
        easy.url(self.url)?;
        easy.post(true)?;

        let mut payload = self.payload.unwrap_or("").as_bytes();
        easy.post_field_size(payload.len() as u64)?;

        let mut output = Vec::new();
        {
            let mut transfer = easy.transfer();
            transfer.read_function(|buf| {
                Ok(payload.read(buf).unwrap_or(0))
            })?;

            transfer.write_function(|data| {
                output.extend_from_slice(data);
                Ok(data.len())
            })?;

            transfer.perform()?;
        }
        Ok(output)
    }
}
