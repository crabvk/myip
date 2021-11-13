use std::net::IpAddr;
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;

fn help_msg() -> String {
    format!(
        "Simple command-line tool to get your external IP address.

Usage:
    myip [options]

Options:
    -a --address <ip>     Look for specified IP
    -l --lookup=[<path>]  Lookup MaxMind database [default path: {}]
    -c --color <when>     When to colorize text output [possible values: {}]
    -j --json             Output in JSON

    -h --help             Prints help information
    -v --version          Prints version information",
        crate::DATABASE_PATH,
        crate::COLORS.join(", ")
    )
}

#[derive(Debug)]
pub struct Args {
    pub address: Option<IpAddr>,
    pub lookup: Option<PathBuf>,
    pub color: Color,
    pub json: bool,
    version: bool,
}

pub fn from_env() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;

    let mut args = Args {
        address: None,
        lookup: None,
        color: Color::Auto,
        json: false,
        version: false,
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
                    args.lookup = Some(PathBuf::from(crate::DATABASE_PATH))
                }
            }
            Short('c') | Long("color") => args.color = parser.value()?.parse()?,
            Short('j') | Long("json") => args.json = true,
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

impl FromStr for Color {
    type Err = String;

    fn from_str(input: &str) -> Result<Color, Self::Err> {
        match input {
            "auto" => Ok(Color::Auto),
            "always" => Ok(Color::Always),
            "never" => Ok(Color::Never),
            _ => Err(format!(
                "Invalid color '{}' [possible values: {}]",
                input,
                crate::COLORS.join(", ")
            )),
        }
    }
}
