mod fastd_key;

use base64::encode as b64_encode;
use clap::{builder::ValueParser, CommandFactory, ErrorKind, Parser};
use env_logger::{fmt::Color, Builder, Env};
use fastd_key::{parse_from_config, parse_from_raw};
use libecdsautil::compressed_points::CompressedLegacyX;
use log::Level;
use log::{error, warn};
use std::fs::File;
use std::io::stdin;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;

/// Translates fastd to WireGuard keys
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Optional public key to translate
    #[clap(value_parser)]
    key: Option<String>,

    /// Input file, or - to read from stdin, (<KEY> and --if are mutually exclusive)
    #[clap(
        short,
        long,
        value_name = "PATH",
        conflicts_with = "key",
        default_value = "-",
        value_parser(ValueParser::new(parse_opt_path))
    )]
    r#if: OptPath,

    /// Output file, or - to write to stdout
    #[clap(
        short,
        long,
        value_name = "PATH",
        default_value = "-",
        value_parser(ValueParser::new(parse_opt_path))
    )]
    of: OptPath,

    /// Whether the given key is a secret key (<KEY> and --private are mutually exclusive)
    #[clap(short, long, action, visible_alias = "secret")]
    private: bool,
}

type OptPath = Option<PathBuf>;
fn parse_opt_path(possible_dash: &str) -> Result<OptPath, std::io::Error> {
    // this is not safe to call on arguments, that have optional values
    if possible_dash == "-" {
        return Ok(None);
    }
    Ok(Some(PathBuf::from(possible_dash)))
}

fn main() {
    let mut builder = Builder::from_env(Env::default().default_filter_or("warn"));
    builder
        .format(|buf, record| {
            let mut style = buf.style();
            let col = match record.level() {
                Level::Error => Color::Red,
                Level::Warn => Color::Yellow,
                _ => Color::White,
            };
            style.set_color(col).set_bold(true);
            writeln!(
                buf,
                "{}: {}",
                style.value(str::to_lowercase(record.level().as_str())),
                record.args()
            )
        })
        .init();

    let args = Args::parse();

    let (private, input_file, public_key) = (args.private, args.r#if, args.key);

    let opt_key_bytes: Option<[u8; 32]> = match (private, input_file, public_key) {
        (false, None, Some(public)) => parse_from_raw(&public),
        (true, None, Some(_)) => {
            let mut cmd = Args::command();
            warn!("Possible pollution of shell history and/or logs with a private key.");
            cmd.error(
                ErrorKind::ArgumentConflict,
                "Private keys cannot be provided as argument but have to be read from file or stdin.",
            ).exit();
        }
        (_, Some(p), None) => {
            let file = match File::open(&p) {
                Ok(file) => file,
                Err(_) => {
                    let variant = match private {
                        true => "private",
                        false => "public",
                    };
                    error!("Unable to read {} key from {:?}", variant, &p);
                    exit(1);
                }
            };
            BufReader::new(file)
                .lines()
                .filter_map(Result::ok)
                .filter_map(|line| parse_from_raw(&line).or_else(|| parse_from_config(&line)))
                .next()
        }
        (_, None, None) => parse_from_raw(&stdin().lock().lines().next().unwrap().unwrap()),
        (_, Some(_), Some(_)) => {
            unimplemented!("This should've been caught by clap due to `conflicts_with = \"key\",`")
        }
    };

    let key_bytes = match opt_key_bytes {
        Some(bytes) => bytes,
        None => {
            error!("Invalid KeyFormat: Unable to decode hex.");
            exit(1);
        }
    };

    let result_bytes = match args.private {
        true => key_bytes,
        false => {
            let fastd_pubkey = CompressedLegacyX(key_bytes);
            let packed_ed25519 = fastd_pubkey.to_compressed_edwards_x();
            let decomp = match packed_ed25519.decompress() {
                Some(d) => d,
                None => {
                    error!("Given key does not represent a valid X-coordinate on Edwards25519.");
                    exit(1);
                }
            };
            let mont = decomp.to_montgomery();
            mont.to_bytes()
        }
    };

    match args.of {
        None => println!("{}", b64_encode(&result_bytes)),
        Some(of_path) => {
            let display = of_path.display();

            let mut file = match File::create(&of_path) {
                Err(reason) => {
                    error!("Could not create {}: {}.", display, reason);
                    exit(1);
                }
                Ok(file) => file,
            };

            if let Err(reason) = file.write_all((b64_encode(&result_bytes) + "\n").as_bytes()) {
                error!("Could not write to {}: {}", display, reason);
                exit(1);
            }
        }
    }
    exit(0);
}
