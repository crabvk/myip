use super::db_error::{error_message as db_error_message, error_type as db_error_type};
use super::dns_error::{error_message as dns_error_message, error_type as dns_error_type};
use super::geoip::GeoIpData;
use ansi_term::Color::{Blue, Green};
use ansi_term::Style;
use dns_transport::Error as TransportError;
use json_minimal::Json;
use maxminddb::MaxMindDBError;
use std::char;

pub enum OutputFormat {
    Text(bool),
    Json,
}

impl OutputFormat {
    pub fn format(&self, data: &mut GeoIpData) -> String {
        match self {
            Self::Text(use_color) => TextData::new(*use_color)
                .add_some(data, "IP", "ip")
                .add_some(data, "City", "city")
                .add_pair(data, "Country", "country", "country_code")
                .add_pair(data, "Region", "region", "region_code")
                .add_pair(data, "Registered", "registered", "registered_code")
                .add_some(data, "ISP", "isp")
                .add_some(data, "Time zone", "time_zone")
                .print(),
            Self::Json => {
                let jd = JsonData::new()
                    .add_some(data, "IP", "ip")
                    .add_some(data, "city", "city")
                    .add_some(data, "region", "region")
                    .add_some(data, "regionCode", "region_code")
                    .add_some(data, "registered", "registered")
                    .add_some(data, "timeZone", "time_zone")
                    .add_some(data, "ISP", "isp");

                match data.remove("country") {
                    Some(name) => {
                        let code = data.remove("country_code").unwrap();
                        let flag = country_code_to_flag(&code);
                        jd.add("country", name)
                            .add("countryCode", code)
                            .add("flag", flag)
                            .print()
                    }
                    _ => jd.print(),
                }
            }
        }
    }

    pub fn format_dns_error(&self, error: TransportError) -> String {
        match self {
            OutputFormat::Text(_) => {
                format!(
                    "Error [{}]: {}",
                    dns_error_type(&error),
                    dns_error_message(error)
                )
            }

            OutputFormat::Json => JsonData::new()
                .add("type", dns_error_type(&error))
                .add("error", dns_error_message(error))
                .print(),
        }
    }

    pub fn format_db_error(&self, error: MaxMindDBError) -> String {
        match self {
            OutputFormat::Text(_) => {
                format!(
                    "Error [{}]: {}",
                    db_error_type(&error),
                    db_error_message(error)
                )
            }

            OutputFormat::Json => JsonData::new()
                .add("type", db_error_type(&error).into())
                .add("error", db_error_message(error).into())
                .print(),
        }
    }
}

struct TextData {
    lines: Vec<String>,
    use_color: bool,
}

impl TextData {
    fn new(use_color: bool) -> Self {
        Self {
            lines: vec![],
            use_color,
        }
    }

    fn add(mut self, name: &str, value: String) -> Self {
        let i = 10 - name.len();

        let s = if self.use_color {
            format!(
                "{:indent$}{}{} {}",
                "",
                Blue.bold().paint(name),
                Style::new().bold().paint(":"),
                Green.paint(value),
                indent = i
            )
        } else {
            format!("{:indent$}{}: {}", "", name, value, indent = i)
        };

        self.lines.push(s);
        self
    }

    fn add_some(self, data: &mut GeoIpData, name: &str, key: &str) -> Self {
        match data.remove(key) {
            Some(value) => self.add(name, value),
            None => self,
        }
    }

    fn add_pair(self, data: &mut GeoIpData, name: &str, key1: &str, key2: &str) -> Self {
        match data.remove(key1) {
            Some(value1) => {
                let value2 = data.remove(key2).unwrap();
                let value = format!("{} ({})", value1, value2);
                self.add(name, value)
            }
            _ => self,
        }
    }

    fn print(self) -> String {
        self.lines.join("\n")
    }
}

struct JsonData {
    json: Json,
}

impl JsonData {
    fn new() -> Self {
        Self { json: Json::new() }
    }

    fn add(mut self, name: &str, value: String) -> Self {
        self.json.add(Json::OBJECT {
            name: name.into(),
            value: Box::new(Json::STRING(value)),
        });
        self
    }

    fn add_some(self, data: &mut GeoIpData, name: &str, key: &str) -> Self {
        match data.remove(key) {
            Some(value) => self.add(name, value),
            _ => self,
        }
    }

    fn print(self) -> String {
        self.json.print()
    }
}

// https://stackoverflow.com/a/42235254/1878180
fn country_code_to_flag(iso_code: &str) -> String {
    let offset: u32 = 0x1F1A5;
    let mut country = iso_code.bytes();
    let char0 = country.next().unwrap() as u32 + offset;
    let char1 = country.next().unwrap() as u32 + offset;

    format!(
        "{}{}",
        char::from_u32(char0).unwrap(),
        char::from_u32(char1).unwrap()
    )
}
