mod db_error;
mod dns_error;
mod geoip;
mod ip;
mod output;

use atty::Stream;
use clap::{App, AppSettings, Arg};
use output::OutputFormat;
use std::net::{IpAddr, Ipv4Addr};
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
        .arg(Arg::from_usage("-i, --ip=[IP] 'Look for specified IP.'").validator(is_ipv4))
        .arg(
            Arg::from_usage("-s, --short 'Show only ip (without quering MaxMind database).'")
                .conflicts_with_all(&["ip", "db", "json", "color"]),
        )
        .arg(Arg::from_usage("-j, --json 'Output in JSON.'"))
        .arg(Arg::from_usage(
            "-d, --db=[PATH] 'Custom MaxMind database directory [default: /var/lib/GeoIP].'",
        ))
        .arg(
            Arg::from_usage(
                "-c, --color 'When to colorize text output (always, never, auto) [default: auto].'",
            )
            .possible_values(&["always", "never", "auto"])
            .conflicts_with("json"),
        )
        .get_matches();

    let out = if matches.is_present("json") {
        OutputFormat::Json
    } else if matches.is_present("short") {
        OutputFormat::Text(false)
    } else {
        let color = matches.value_of("color").unwrap_or("auto");
        let use_color = match color {
            "always" => true,
            "never" => false,
            _ => atty::is(Stream::Stdout),
        };

        OutputFormat::Text(use_color)
    };

    let addr = match matches.value_of("ip") {
        Some(addr) => IpAddr::V4(addr.parse().unwrap()),
        None => ip::dig().unwrap_or_else(|e| {
            eprintln!("{}", out.format_dns_error(e));
            exit(1)
        }),
    };

    if matches.is_present("short") {
        println!("{}", addr);
        exit(0);
    }

    let path = matches.value_of("db").unwrap_or(DEFAULT_DATABASE_PATH);
    let mut data = geoip::lookup(path, addr).unwrap_or_else(|e| {
        eprintln!("{}", out.format_db_error(e));
        exit(2)
    });

    println!("{}", out.format(&mut data));
}
