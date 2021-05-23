use ansi_term::Color::{Blue, Green};
use ansi_term::Style;
use maxminddb::geoip2::{Asn, City};
use std::char;
use std::collections::HashMap;
use std::net::IpAddr;

pub type GeoIpData<'a> = HashMap<&'a str, &'a str>;

pub enum Format {
    Text(bool),
    Json,
}

impl Format {
    pub fn output<'a>(&self, ip: IpAddr, city: &'a City, asn: &'a Asn) -> String {
        match self {
            Self::Text(use_colors) => {
                let data = &mut get_data(city, asn);
                TextOutput::new(*use_colors)
                    .add_some(data, "IP", "ip")
                    .add_some(data, "City", "city")
                    .add_pair(data, "Country", "country", "country_code")
                    .add_pair(data, "Region", "region", "region_code")
                    .add_pair(data, "Registered", "registered", "registered_code")
                    .add_some(data, "ISP", "isp")
                    .add_some(data, "Time zone", "time_zone")
                    .to_string()
            }
            Self::Json => {
                let mut data = get_data(city, asn);
                let ip = &ip.to_string();
                data.insert("ip", ip);

                let flag;
                if let Some(code) = data.get("country_code") {
                    flag = get_flag(&code);
                    data.insert("flag", &flag);
                }

                serde_json::to_string(&data).unwrap()
            }
        }
    }

    pub fn format_error(&self, msg: String, ip: Option<String>) -> String {
        match self {
            Self::Text(_) => {
                if let Some(ip) = ip {
                    let mut error = ip;
                    error.push('\n');
                    error.push_str(&msg);
                    error
                } else {
                    msg
                }
            }
            Self::Json => {
                let mut error = HashMap::new();
                error.insert("error", msg);
                if let Some(ip) = ip {
                    error.insert("ip", ip);
                }
                serde_json::to_string(&error).unwrap()
            }
        }
    }
}

fn get_data<'a>(city: &'a City, asn: &'a Asn) -> GeoIpData<'a> {
    let mut data: GeoIpData = HashMap::new();

    if let Some(city) = &city.city {
        data.insert("city", city.names.as_ref().unwrap()["en"]);
    }

    if let Some(country) = &city.country {
        data.insert("country", country.names.as_ref().unwrap()["en"]);
        data.insert("country_code", country.iso_code.unwrap());
    }

    if let Some(sds) = &city.subdivisions {
        let region = sds.first().unwrap();
        data.insert("region", region.names.as_ref().unwrap()["en"]);
        data.insert("region_code", region.iso_code.unwrap());
    }

    if let Some(registered) = &city.registered_country {
        data.insert("registered", registered.names.as_ref().unwrap()["en"]);
        data.insert("registered_code", registered.iso_code.unwrap());
    }

    if let Some(location) = &city.location {
        data.insert("time_zone", location.time_zone.unwrap());
    }

    if let Some(isp) = asn.autonomous_system_organization {
        data.insert("isp", isp);
    }

    data
}

struct TextOutput {
    lines: Vec<String>,
    use_color: bool,
}

impl TextOutput {
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
            Some(value) => self.add(name, value.into()),
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

    fn to_string(self) -> String {
        self.lines.join("\n")
    }
}

// https://stackoverflow.com/a/42235254/1878180
fn get_flag(iso_code: &str) -> String {
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
