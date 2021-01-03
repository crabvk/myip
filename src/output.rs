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
    Text,
    Json,
}

impl OutputFormat {
    pub fn format(&self, data: GeoIpData) -> String {
        match self {
            Self::Text => TextData::new()
                .add_field("IP", data.ip)
                .add_some("City", data.city)
                .add_pair("Country", data.country)
                .add_pair("Region", data.region)
                .add_pair("Registered", data.registered)
                .add_some("ISP", data.isp)
                .add_some("Time zone", data.time_zone)
                .print(),
            Self::Json => {
                let jd = JsonData::new()
                    .add_field("IP", data.ip)
                    .add_some("city", data.city)
                    .add_pair("region", "regionCode", data.region)
                    .add_pair("registered", "registeredCode", data.registered)
                    .add_some("timeZone", data.time_zone)
                    .add_some("ISP", data.isp);

                match data.country {
                    Some((name, code)) => {
                        let flag = country_code_to_flag(&code);
                        jd.add_field("country", name)
                            .add_field("countryCode", code)
                            .add_field("flag", flag)
                            .print()
                    }
                    _ => jd.print(),
                }
            }
        }
    }

    pub fn format_dns_error(&self, error: TransportError) -> String {
        match self {
            OutputFormat::Text => {
                format!(
                    "Error [{}]: {}",
                    dns_error_type(&error),
                    dns_error_message(error)
                )
            }

            OutputFormat::Json => JsonData::new()
                .add_field("type", dns_error_type(&error))
                .add_field("error", dns_error_message(error))
                .print(),
        }
    }

    pub fn format_db_error(&self, error: MaxMindDBError) -> String {
        match self {
            OutputFormat::Text => {
                format!(
                    "Error [{}]: {}",
                    db_error_type(&error),
                    db_error_message(error)
                )
            }

            OutputFormat::Json => JsonData::new()
                .add_field("type", db_error_type(&error).into())
                .add_field("error", db_error_message(error).into())
                .print(),
        }
    }
}

struct TextData {
    lines: Vec<String>,
}

impl TextData {
    fn new() -> Self {
        Self { lines: vec![] }
    }

    fn add_field(mut self, name: &str, value: String) -> Self {
        let s = format!(
            "{:indent$}{}{} {}",
            "",
            Blue.bold().paint(name),
            Style::new().bold().paint(":"),
            Green.paint(value),
            indent = 10 - name.len()
        );
        self.lines.push(s);
        self
    }

    fn add_some(self, name: &str, value: Option<String>) -> Self {
        match value {
            Some(value) => self.add_field(name, value),
            _ => self,
        }
    }

    fn add_pair(self, name: &str, values: Option<(String, String)>) -> Self {
        match values {
            Some((value1, value2)) => {
                let value = format!("{} ({})", value1, value2);
                self.add_field(name, value)
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

    fn add_field(mut self, name: &str, value: String) -> Self {
        self.json.add(Json::OBJECT {
            name: name.into(),
            value: Box::new(Json::STRING(value)),
        });
        self
    }

    fn add_some(self, name: &str, value: Option<String>) -> Self {
        match value {
            Some(value) => self.add_field(name, value),
            _ => self,
        }
    }

    fn add_pair(self, name1: &str, name2: &str, values: Option<(String, String)>) -> Self {
        match values {
            Some((value1, value2)) => self.add_field(name1, value1).add_field(name2, value2),
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
