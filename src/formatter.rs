use ansi_term::Color::{Blue, Green};
use ansi_term::Style;
use maxminddb::geoip2::{Asn, City};
use std::char;
use std::collections::HashMap;
use std::net::IpAddr;

pub struct GeoIpData<'a> {
    city: Option<&'a str>,
    country: Option<(&'a str, &'a str)>,
    region: Option<(&'a str, &'a str)>,
    registred: Option<(&'a str, &'a str)>,
    isp: Option<&'a str>,
    time_zone: Option<&'a str>,
}

impl<'a> GeoIpData<'a> {
    fn new() -> Self {
        Self {
            city: None,
            country: None,
            region: None,
            registred: None,
            isp: None,
            time_zone: None,
        }
    }
}

pub trait Formatter {
    fn format(&self, ip: IpAddr, data: GeoIpData) -> String;
}

pub struct TextFormatter(pub bool);

impl TextFormatter {
    fn format_pair(&self, name: &str, value: &str) -> String {
        let indent = 10 - name.len();

        if self.0 {
            format!(
                "{:indent$}{}{} {}",
                "",
                Blue.bold().paint(name),
                Style::new().bold().paint(":"),
                Green.paint(value),
                indent = indent
            )
        } else {
            format!("{:indent$}{}: {}", "", name, value, indent = indent)
        }
    }
}

impl Formatter for TextFormatter {
    fn format(&self, ip: IpAddr, data: GeoIpData) -> String {
        let mut lines = vec![];

        lines.push(self.format_pair("IP", ip.to_string().as_str()));
        if let Some(city) = data.city {
            lines.push(self.format_pair("City", city));
        }
        if let Some((country, country_code)) = data.country {
            let value = format!("{} ({})", country, country_code);
            lines.push(self.format_pair("Country", &value));
        }
        if let Some((region, region_code)) = data.region {
            let value = format!("{} ({})", region, region_code);
            lines.push(self.format_pair("Region", &value));
        }
        if let Some((registred, registred_code)) = data.registred {
            let value = format!("{} ({})", registred, registred_code);
            lines.push(self.format_pair("Registered", &value));
        }
        if let Some(isp) = data.isp {
            lines.push(self.format_pair("ISP", isp));
        }
        if let Some(time_zone) = data.time_zone {
            lines.push(self.format_pair("Time zone", time_zone));
        }

        lines.join("\n")
    }
}

pub struct JsonFormatter;

impl Formatter for JsonFormatter {
    fn format(&self, ip: IpAddr, data: GeoIpData) -> String {
        let ip = ip.to_string();
        let mut hm = HashMap::new();
        let flag;

        hm.insert("IP", ip.as_str());
        if let Some(city) = data.city {
            hm.insert("City", city);
        }
        if let Some((country, country_code)) = data.country {
            hm.insert("Country", country);
            hm.insert("CountryCode", country_code);
            flag = get_flag(&country_code);
            hm.insert("Flag", &flag);
        }
        if let Some((region, region_code)) = data.region {
            hm.insert("Region", region);
            hm.insert("RegionCode", region_code);
        }
        if let Some((registred, registred_code)) = data.registred {
            hm.insert("Registered", registred);
            hm.insert("RegisteredCode", registred_code);
        }
        if let Some(isp) = data.isp {
            hm.insert("ISP", isp);
        }
        if let Some(time_zone) = data.time_zone {
            hm.insert("TimeZone", time_zone);
        }

        serde_json::to_string(&hm).unwrap()
    }
}

pub fn map_data<'a>(city: City<'a>, asn: Asn<'a>) -> GeoIpData<'a> {
    let mut data = GeoIpData::new();

    data.city = city.city.map(|city| city.names.as_ref().unwrap()["en"]);
    data.country = city.country.map(|country| {
        let name = country.names.as_ref().unwrap()["en"];
        let code = country.iso_code.unwrap();
        (name, code)
    });
    data.region = city.subdivisions.map(|subdiv| {
        let region = subdiv.first().unwrap();
        let name = region.names.as_ref().unwrap()["en"];
        let code = region.iso_code.unwrap();
        (name, code)
    });
    city.registered_country.map(|reg| {
        let name = reg.names.as_ref().unwrap()["en"];
        let code = reg.iso_code.unwrap();
        (name, code)
    });
    data.time_zone = city.location.map(|location| location.time_zone.unwrap());
    data.isp = asn.autonomous_system_organization.map(|isp| isp);
    data
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
