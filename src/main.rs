mod dns_error;
mod ip;
mod output;

use atty::Stream;
use clap::{App, AppSettings, Arg};
use dns_error::error_message;
use maxminddb::{MaxMindDBError, Reader};
use output::Format;
use std::net::{IpAddr, Ipv4Addr};
use std::path::Path;
use std::process::exit;

const DEFAULT_DATABASE_PATH: &str = "/var/lib/GeoIP";

fn is_ipv4(value: String) -> Result<(), String> {
    if value.parse::<Ipv4Addr>().is_ok() {
        Ok(())
    } else {
        Err(String::from("specified IPv4 address has wrong format."))
    }
}

fn main() {
    let matches = App::new("myip")
        .set_term_width(0)
        .setting(AppSettings::DeriveDisplayOrder)
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .version_short("v")
        .arg(Arg::from_usage("-a, --address=[IP] 'Look for specified IP'").validator(is_ipv4))
        .arg(
            Arg::from_usage("-s, --short 'Show only IP (skip query to local MaxMind database)'")
                .conflicts_with_all(&["ip", "db", "json", "color"]),
        )
        .arg(Arg::from_usage("-j, --json 'Output in JSON'"))
        .arg(
            Arg::from_usage("-d, --db=[PATH] 'Custom MaxMind database directory'")
                .default_value(DEFAULT_DATABASE_PATH),
        )
        .arg(
            Arg::from_usage("-c, --color=[WHEN] 'When to colorize text output'")
                .default_value("auto")
                .possible_values(&["always", "never", "auto"])
                .conflicts_with("json"),
        )
        .get_matches();

    let format = if matches.is_present("json") {
        Format::Json
    } else if matches.is_present("short") {
        Format::Text(false)
    } else {
        let color = matches.value_of("color").unwrap_or_default();
        let use_color = match color {
            "always" => true,
            "never" => false,
            _ => atty::is(Stream::Stdout),
        };

        Format::Text(use_color)
    };

    let addr = match matches.value_of("address") {
        Some(addr) => IpAddr::V4(addr.parse().unwrap()),
        None => ip::dig().unwrap_or_else(|error| {
            let msg = error_message(error);
            eprintln!("{}", format.format_error(msg, None));
            exit(1)
        }),
    };

    if matches.is_present("short") {
        println!("{}", addr);
        return;
    }

    let path = matches.value_of("db").unwrap_or_default();
    let output = query_db(&format, path, addr).unwrap_or_else(|error| {
        let msg = error.to_string();
        let ip = addr.to_string();
        eprintln!("{}", format.format_error(msg, Some(ip)));
        exit(2)
    });

    println!("{}", output);
}

fn query_db(format: &Format, path: &str, addr: IpAddr) -> Result<String, MaxMindDBError> {
    let reader = load_database(path, "GeoLite2-City.mmdb")?;
    let city = reader.lookup(addr)?;

    let reader = load_database(path, "GeoLite2-ASN.mmdb")?;
    let asn = reader.lookup(addr)?;
    Ok(format.output(addr, &city, &asn))
}

pub fn load_database(path: &str, filename: &str) -> Result<Reader<Vec<u8>>, MaxMindDBError> {
    let path = Path::new(path).join(filename);
    Reader::open_readfile(path)
}
