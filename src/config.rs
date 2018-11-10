use errors::AppError;
use ini::Ini;


pub struct ConfigParser<'a> {
    conf: &'a Ini,
}

impl<'a> ConfigParser<'a> {
    pub fn new(conf: &'a Ini) -> Self {
        ConfigParser{conf}
    }

    pub fn section<'b: 'a>(&self, section: &'b str) -> SectionParser<'a, 'b> {
        SectionParser{
            conf: self.conf,
            section,
        }
    }
}

pub struct SectionParser<'a, 'b> {
    conf: &'a Ini,
    section: &'b str,
}

impl<'a, 'b> SectionParser<'a, 'b> {
    pub fn get(&self, key: &str) -> Result<&'a str, AppError> {
        self.conf.get_from(Some(self.section), key)
            .ok_or(AppError::Config(format!("Missing parameter '{}' in section '{}'", key, self.section)))
    }

    pub fn get_or(&self, key: &str, default: &'a str) -> &'a str {
        match self.conf.get_from(Some(self.section), key) {
            Some(value) => value,
            _           => default
        }
    }

    pub fn get_or_none(&self, key: &str) -> Option<&'a str> {
        self.conf.get_from(Some(self.section), key)
    }
}
