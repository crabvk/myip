mod argparser;
mod formatter;
mod resolver;

use crate::argparser::Color;
use crate::formatter::{map_data, Formatter, JsonFormatter, TextFormatter};
use crate::resolver::IpAddrKind;
use atty::Stream;
use maxminddb::{
    geoip2::{Asn, City},
    MaxMindDBError, Reader,
};
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;
use std::process::exit;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const MMDB_CITY_FILENAME: &str = "GeoLite2-City.mmdb";
const MMDB_ASN_FILENAME: &str = "GeoLite2-ASN.mmdb";
const MMDB_PATH: &str = "/var/lib/GeoIP";
const COLORS: [&str; 3] = ["auto", "always", "never"];

fn main() -> Result<(), lexopt::Error> {
    let args = argparser::from_env()?;

    let ip = if let Some(address) = args.address {
        address
    } else {
        let ip_kind = if args.inet6 {
            IpAddrKind::V6
        } else {
            IpAddrKind::V4
        };
        resolver::dig(ip_kind).unwrap_or_else(|error| {
            eprintln!("Failed to resolve IP address: {error}");
            exit(1);
        })
    };

    if args.lookup.is_none() {
        if args.json {
            let mut hm = HashMap::new();
            hm.insert("IP", ip);
            let json = serde_json::to_string(&hm).unwrap();
            println!("{json}");
        } else {
            println!("{ip}");
        }
        exit(0);
    }

    let mmdb_path = args.lookup.unwrap();
    let output = if args.json {
        lookup_mmdb(&mmdb_path, ip, JsonFormatter)
    } else {
        let use_color = match args.color {
            Color::Always => true,
            Color::Never => false,
            Color::Auto => atty::is(Stream::Stdout),
        };
        lookup_mmdb(&mmdb_path, ip, TextFormatter(use_color))
    };
    match output {
        Ok(text) => println!("{text}"),
        Err(error) => {
            let msg = format!("Failed to lookup MaxMind GeoIP2 database: {error}");
            if args.json {
                let mut hm = HashMap::new();
                hm.insert("IP", ip.to_string());
                hm.insert("Error", msg);
                let json = serde_json::to_string(&hm).unwrap();
                println!("{json}");
            } else {
                println!("{ip}");
                eprintln!("{msg}")
            }
        }
    }

    Ok(())
}

fn lookup_mmdb(
    path: &PathBuf,
    ip: IpAddr,
    formatter: impl Formatter,
) -> Result<String, MaxMindDBError> {
    let full_path = path.join(MMDB_CITY_FILENAME);
    let reader = Reader::open_readfile(full_path)?;
    let city: City = reader.lookup(ip)?;

    let full_path = path.join(MMDB_ASN_FILENAME);
    let reader = Reader::open_readfile(full_path)?;
    let asn: Asn = reader.lookup(ip)?;

    let data = map_data(city, asn);
    Ok(formatter.format(ip, data))
}
