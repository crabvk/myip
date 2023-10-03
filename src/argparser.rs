use std::net::IpAddr;
use std::path::PathBuf;
use std::process::exit;
use std::{error, fmt, str::FromStr};

fn help_msg() -> String {
    format!(
        "{}

Usage:
    myip [options]

Options:
    -l --lookup=[<path>]  Lookup MaxMind database [default path: {}]
    -a --address <ip>     Look for specified IP
    -c --color <when>     When to colorize text output [possible values: {}]
    -6 --inet6            Use inet6 protocol family (IPv6)
    -j --json             Output in JSON

    -h --help             Prints help information
    -v --version          Prints version information",
        env!("CARGO_PKG_DESCRIPTION"),
        crate::MMDB_PATH,
        crate::COLORS.join(", ")
    )
}

#[derive(Debug)]
pub struct Args {
    pub address: Option<IpAddr>,
    pub lookup: Option<PathBuf>,
    pub color: Color,
    pub inet6: bool,
    pub json: bool,
}

pub fn from_env() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;

    let mut args = Args {
        address: None,
        lookup: None,
        color: Color::Auto,
        inet6: false,
        json: false,
    };

    let mut parser = lexopt::Parser::from_env();

    while let Some(arg) = parser.next()? {
        match arg {
            Short('a') | Long("address") if args.address.is_none() => {
                args.address = Some(parser.value()?.parse()?)
            }
            Short('l') | Long("lookup") => {
                if let Some(path) = parser.optional_value() {
                    args.lookup = Some(path.into_string()?.into())
                } else {
                    args.lookup = Some(PathBuf::from(crate::MMDB_PATH))
                }
            }
            Short('c') | Long("color") => args.color = parser.value()?.parse()?,
            Short('j') | Long("json") => args.json = true,
            Short('6') | Long("inet6") => args.inet6 = true,
            Short('v') | Long("version") => {
                println!("{} {}", crate::NAME, crate::VERSION);
                exit(0);
            }
            Short('h') | Long("help") => {
                println!("{}", help_msg());
                exit(0);
            }
            _ => return Err(arg.unexpected()),
        }
    }

    Ok(args)
}

#[derive(Debug, PartialEq)]
pub enum Color {
    Auto,
    Always,
    Never,
}

#[derive(Debug)]
pub struct ParseColorError(String);

impl fmt::Display for ParseColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown variant \"{}\", possible values: {}",
            self.0,
            crate::COLORS.join(", ")
        )
    }
}

impl error::Error for ParseColorError {}

impl FromStr for Color {
    type Err = ParseColorError;

    fn from_str(input: &str) -> Result<Color, Self::Err> {
        match input {
            "auto" => Ok(Color::Auto),
            "always" => Ok(Color::Always),
            "never" => Ok(Color::Never),
            color => Err(ParseColorError(color.to_owned())),
        }
    }
}
