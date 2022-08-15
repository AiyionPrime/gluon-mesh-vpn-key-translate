use base64::encode as b64_encode;
use clap::{CommandFactory, ErrorKind, Parser};
use env_logger::{fmt::Color, Builder, Env};
use hex::FromHex;
use libecdsautil::compressed_points::CompressedLegacyX;
use log::Level;
use log::{error, warn};
use std::fs::File;
use std::io::stdin;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
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
        parse(from_os_str)
    )]
    r#if: std::path::PathBuf,

    /// Output file, or - to write to stdout
    #[clap(
        short,
        long,
        value_name = "PATH",
        default_value = "-",
        parse(from_os_str)
    )]
    of: std::path::PathBuf,

    /// Whether the given key is a secret key (<KEY> and --private are mutually exclusive)
    #[clap(short, long, action, visible_alias = "secret")]
    private: bool,
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

    //TODO make a TypedValueParser from this
    let clean_if_path = match args.r#if {
        p if p == Path::new("-") => None,
        p => Some(p),
    };

    let clean_of_path = match args.of {
        p if p == Path::new("-") => None,
        p => Some(p),
    };

    let (private, input_file, public_key) = (args.private, clean_if_path, args.key);

    let opt_key_bytes: Result<[u8; 32], hex::FromHexError> = match (private, input_file, public_key)
    {
        (false, None, Some(public)) => <[u8; 32]>::from_hex(public),
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
            let mut buffer = BufReader::new(file);
            let mut first_line = String::new();
            let _ = buffer.read_line(&mut first_line);
            <[u8; 32]>::from_hex(first_line.trim())
        }
        (_, None, None) => {
            <[u8; 32]>::from_hex(stdin().lock().lines().next().unwrap().unwrap().trim())
        }
        (_, Some(_), Some(_)) => {
            unimplemented!("This should've been caught by clap due to `conflicts_with = \"key\",`")
        }
    };

    let key_bytes = match opt_key_bytes {
        Ok(bytes) => bytes,
        Err(_) => {
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
    match clean_of_path {
        None => println!("{}", b64_encode(&result_bytes)),
        Some(of_path) => {
            let display = of_path.display();

            let mut file = match File::create(&of_path) {
                Err(reason) => {
                    error!("Could not create {}: {}.", display, reason);
                    exit(1)
                }
                Ok(file) => file,
            };

            match file.write_all((b64_encode(&result_bytes) + "\n").as_bytes()) {
                Err(reason) => {
                    error!("Could not write to {}: {}", display, reason);
                    exit(1);
                }
                Ok(_) => (),
            }
        }
    }
    exit(0);
}
