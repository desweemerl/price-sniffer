use errors::AppError;
use std::fs::File;
use std::io::prelude::*;

use yaml_rust::Yaml;
use yaml_rust::yaml::Hash;
use yaml_rust::yaml::YamlLoader;


fn error(msg: &str) -> AppError {
    AppError::Config(String::from(msg))
}

pub struct ConfigParser(Yaml);

impl ConfigParser {
    pub fn new<'a>(filename: &'a str) -> Result<Self, AppError> {
        let mut file = File::open(filename)?;
        let mut config_string = String::new();
        file.read_to_string(&mut config_string)?;

        YamlLoader::load_from_str(&config_string)?
            .into_iter().nth(0)
            .map(|config| ConfigParser(config))
            .ok_or(error(&format!("Cannot read configuration file '{}'", filename)))
    }

    pub fn section<'a>(self, name: &'a str) -> SectionParser {
        self.0.as_hash()
              .and_then(|config| config.get(&Yaml::from_str(name)))
              .map_or(
                  SectionParser{conf: None, name},
                  |section| SectionParser{conf: Some(section.clone()), name}
              )
    }
}

impl Clone for ConfigParser{
    fn clone(&self) -> Self {
        ConfigParser(self.0.clone())
    }
}

pub struct SectionParser<'a>{
    conf: Option<Yaml>,
    name: &'a str,
}

impl<'a> SectionParser<'a> {
    fn get(&self, key: &str) -> Result<Yaml, AppError> {
        self.conf.as_ref()
                 .and_then(|conf| conf.as_hash())
                 .ok_or(error(&format!("Section '{}' is not a hash", self.name)))?
                 .get(&Yaml::from_str(key))
                 .map(|v| v.clone())
                 .ok_or(error(&format!("Missing key '{}' in section '{}'", key, self.name)))
    }

    pub fn get_str(&self, key: &str) -> Result<String, AppError> {
        let value = self.get(key)?;

        value.as_str()
             .ok_or(error(&format!("Key '{}' in Section '{}' is not a string", key, self.name)))
             .map(|v| String::from(v))
    }

    pub fn get_str_or(&self, key: &str, default: &str) -> String {
        self.get_str(key).unwrap_or(String::from(default))
    }

    pub fn get_str_or_none(&self, key: &str) -> Option<String> {
        self.get_str(key).ok()
    }

    pub fn get_hash(&self, key: &str) -> Result<Hash, AppError> {
        let value = self.get(key)?;

        value.as_hash()
             .ok_or(error(&format!("Key '{}' in Section '{}' is not a hash", key, self.name)))
             .map(|v| v.clone())
    }

    pub fn get_hash_or_none(&self, key: &str) -> Option<Hash> {
        self.get_hash(key).ok()
    }
    #[allow(dead_code)] 
    pub fn get_i64(&self, key: &str) -> Result<i64, AppError> {
        self.get(key)
            .and_then(|value| value.as_i64().ok_or(error(&format!("Value '{}' in section '{}' must be an integer", key, self.name))))
    }

    #[allow(dead_code)] 
    pub fn get_i64_or(&self, key: &str, default: i64) -> i64 {
        self.get_i64(key).unwrap_or(default)
    }

    #[allow(dead_code)] 
    pub fn get_i64_or_none(&self, key: &str) -> Option<i64> {
        self.get_i64(key).ok()
    }
}
