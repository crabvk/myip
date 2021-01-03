mod db_error;
mod dns_error;
mod geoip;
mod ip;
mod output;

use clap::{App, AppSettings, Arg};
use output::OutputFormat;
use std::process::exit;

const DEFAULT_DATABASE_PATH: &str = "/var/lib/GeoIP";

fn main() {
    let matches = App::new("myip")
        .set_term_width(0)
        .setting(AppSettings::DeriveDisplayOrder)
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .version_short("v")
        .arg(
            Arg::from_usage("-i, --ip 'Show only ip (without quering MaxMind database).'")
                .conflicts_with_all(&["db", "json"]),
        )
        .arg(Arg::from_usage("-j, --json 'Output in JSON.'"))
        .arg(Arg::from_usage(
            "-d, --db=[PATH] 'Custom MaxMind database directory [default: /var/lib/GeoIP].'",
        ))
        .get_matches();

    let out = if matches.is_present("json") && !matches.is_present("ip") {
        OutputFormat::Json
    } else {
        OutputFormat::Text
    };

    let addr = ip::dig().unwrap_or_else(|e| {
        eprintln!("{}", out.format_dns_error(e));
        exit(1)
    });

    if matches.is_present("ip") {
        println!("{}", addr);
        exit(0);
    }

    let path = matches.value_of("db").unwrap_or(DEFAULT_DATABASE_PATH);
    let data = geoip::lookup(path, addr).unwrap_or_else(|e| {
        eprintln!("{}", out.format_db_error(e));
        exit(2)
    });

    println!("{}", out.format(data));
}
