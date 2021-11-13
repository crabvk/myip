mod args_parser;
mod dns_error;
mod ip;
mod output;

use crate::args_parser::Color;
use atty::Stream;
use dns_error::error_message;
use maxminddb::{MaxMindDBError, Reader};
use output::Format;
use std::net::IpAddr;
use std::path::PathBuf;
use std::process::exit;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const MMDB_CITY_FILE: &str = "GeoLite2-City.mmdb";
const MMDB_ASN_FILE: &str = "GeoLite2-ASN.mmdb";
const DATABASE_PATH: &str = "/var/lib/GeoIP";
const COLORS: [&str; 3] = ["auto", "always", "never"];

fn main() -> Result<(), lexopt::Error> {
    let args = args_parser::from_env()?;

    let format = if args.json {
        Format::Json
    } else if args.lookup.is_none() {
        Format::Text(false)
    } else {
        use Color::*;
        let use_color = match args.color {
            Always => true,
            Never => false,
            Auto => atty::is(Stream::Stdout),
        };
        Format::Text(use_color)
    };

    let addr = if let Some(addr) = args.address {
        addr
    } else {
        ip::dig().unwrap_or_else(|error| {
            let msg = error_message(error);
            eprintln!("{}", format.format_error(msg, None));
            exit(1)
        })
    };

    if args.lookup.is_none() {
        println!("{}", addr);
        exit(0);
    }

    let output = lookup_mmdb(&format, &args.lookup.unwrap(), addr).unwrap_or_else(|error| {
        let msg = error.to_string();
        let ip = addr.to_string();
        eprintln!("{}", format.format_error(msg, Some(ip)));
        exit(2)
    });

    println!("{}", output);

    Ok(())
}

fn lookup_mmdb(format: &Format, path: &PathBuf, addr: IpAddr) -> Result<String, MaxMindDBError> {
    let reader = open_mmdb(path, MMDB_CITY_FILE)?;
    let city = reader.lookup(addr)?;

    let reader = open_mmdb(path, MMDB_ASN_FILE)?;
    let asn = reader.lookup(addr)?;
    Ok(format.output(addr, &city, &asn))
}

pub fn open_mmdb(path: &PathBuf, filename: &str) -> Result<Reader<Vec<u8>>, MaxMindDBError> {
    let path = path.join(filename);
    Reader::open_readfile(path)
}
