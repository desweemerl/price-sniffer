use config::ConfigParser;
use errors::AppError;
use rust_decimal::Decimal;
use regex::Regex;

use std::str;
use std::str::FromStr;

use yaml_rust::yaml::Hash;
use yaml_rust::Yaml;

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

pub struct HttpProcessor {
    url: String,
    method: HttpMethod,
    payload: Option<Hash>,
    re: Regex,
}

fn yaml_to_str(value: &Yaml) -> Result<String, ()> {
    match value {
        Yaml::String(s) => Ok(s.clone()),
        Yaml::Integer(ref i) => Ok(i.to_string()),
        _ => Err(()),
    }

}

impl HttpProcessor {

    pub fn from_config<'b>(conf: &'b ConfigParser) -> Result<Self, AppError> {
        let cfg_processing = conf.clone().section("processing");
        let url = cfg_processing.get_str("url")?;
        debug!("url='{}'", url);
        let regex_str = cfg_processing.get_str("regex")?;
        debug!("regex='{}'", regex_str);
        let re = regex::RegexBuilder::new(&regex_str).multi_line(true).build()?;
        let method_str = cfg_processing.get_str_or("method", "GET");
        debug!("method='{}'", method_str);
        let method = HttpMethod::from_str(&method_str)?;
        let payload = cfg_processing.get_hash_or_none("payload");
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
        self.re
            .captures(&response)
            .map_or(
                Ok(None), |cap|
                cap.get(1)
                   .map_or(
                     Ok(None),
                     |m| match Decimal::from_str(m.as_str()) {
                        Ok(price) => Ok(Some(price)),
                        Err(err) => Err(AppError::from(err))
                     }
                   )
                
            )
    }

    fn post(&self) -> Result<String, AppError> {
        let mut form:Vec<(String, String)> = Vec::new();
        if let Some(ref payload) = self.payload {
            for (key, value) in payload {
                form.push((yaml_to_str(key).unwrap(), yaml_to_str(value).unwrap()));
            }
        }
        let mut response = reqwest::Client::new()
            .post(&self.url)
            .form(&form)
            .send()?
            .error_for_status()?;

        Ok(response.text()?)
    }
}
