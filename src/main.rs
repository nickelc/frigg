use std::borrow::Cow;
use std::io;
use std::path::PathBuf;

use clap::{crate_description, crate_name, crate_version};
use clap::{App, AppSettings, Arg};
use futures_util::TryStreamExt;
use tokio::fs::File;
use tokio::io::{BufReader, BufWriter};
use tokio_util::io::StreamReader;

mod auth;
mod binary_info;
mod client;
mod decrypt;
mod progress;
mod requests;
mod version;

use binary_info::{BinaryInfo, DecryptKey};
use client::Client;

type Error = Box<dyn std::error::Error>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let app = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            App::new("check")
                .about("check for the lastest available firmware version")
                .arg(
                    Arg::new("model")
                        .long("model")
                        .short('m')
                        .required(true)
                        .help("device model")
                        .value_name("MODEL"),
                )
                .arg(
                    Arg::new("region")
                        .long("region")
                        .short('r')
                        .required(true)
                        .help("region model")
                        .value_name("REGION"),
                ),
        )
        .subcommand(
            App::new("download")
                .about("download the latest firmware")
                .arg(
                    Arg::new("model")
                        .long("model")
                        .short('m')
                        .required(true)
                        .help("device model")
                        .value_name("MODEL"),
                )
                .arg(
                    Arg::new("region")
                        .long("region")
                        .short('r')
                        .required(true)
                        .help("region model")
                        .value_name("REGION"),
                )
                .arg(
                    Arg::new("download-only")
                        .long("download-only")
                        .help("don't decrypt the firmware file"),
                )
                .arg(
                    Arg::new("output")
                        .allow_invalid_utf8(true)
                        .value_name("OUTPUT")
                        .help("output to a specific file or directory"),
                ),
        )
        .subcommand(
            App::new("decrypt")
                .about("decrypt a downloaded firmware")
                .arg(
                    Arg::new("model")
                        .long("model")
                        .short('m')
                        .required(true)
                        .help("device model")
                        .value_name("MODEL"),
                )
                .arg(
                    Arg::new("region")
                        .long("region")
                        .short('r')
                        .required(true)
                        .help("region model")
                        .value_name("REGION"),
                )
                .arg(
                    Arg::new("version")
                        .long("firmware-version")
                        .short('v')
                        .required(true)
                        .value_name("VERSION"),
                )
                .arg(
                    Arg::new("input")
                        .allow_invalid_utf8(true)
                        .required(true)
                        .value_name("INPUT")
                        .help("path to encrypted firmware"),
                )
                .arg(
                    Arg::new("output")
                        .allow_invalid_utf8(true)
                        .value_name("OUTPUT")
                        .help("output to a specific file or directory"),
                ),
        );

    match app.get_matches().subcommand() {
        Some(("check", matches)) => {
            let model = matches.value_of("model").expect("arg is required");
            let region = matches.value_of("region").expect("arg is required");

            let client = Client::new()?;
            let version = client.fetch_version(&model, &region).await?;
            let mut session = client.begin_session().await?;
            let info = client
                .file_info(&model, &region, &version, &mut session)
                .await?;

            print_info(&model, &region, &info);
        }
        Some(("download", matches)) => {
            let model = matches.value_of("model").expect("arg is required");
            let region = matches.value_of("region").expect("arg is required");

            let output = match matches.value_of_os("output").map(PathBuf::from) {
                Some(output) if output.is_dir() => Some(Destination::Dir(output)),
                Some(output) if !output.exists() => Some(Destination::File(output)),
                Some(_) | None => None,
            };

            let client = Client::new()?;
            let version = client.fetch_version(&model, &region).await?;
            let mut session = client.begin_session().await?;
            let info = client
                .file_info(&model, &region, &version, &mut session)
                .await?;

            print_info(&model, &region, &info);

            let resp = client.download(&info, &mut session).await?;

            let (filename, decrypt_key) = if matches.is_present("download-only") {
                (Cow::from(info.binary_name), None)
            } else {
                match (
                    info.decrypt_key,
                    info.binary_name.strip_suffix(".enc2"),
                    info.binary_name.strip_suffix(".enc4"),
                ) {
                    (DecryptKey::V2(key), Some(filename), None)
                    | (DecryptKey::V4(key), None, Some(filename)) => {
                        (Cow::from(filename), Some(key))
                    }
                    (DecryptKey::Unknown, None, None) => {
                        tracing::warn!(
                            "couldn't determine decryption key. falling back to download only."
                        );
                        (Cow::from(info.binary_name), None)
                    }
                    _ => unreachable!(),
                }
            };

            let pb = progress::new(info.binary_size);

            let st = resp
                .bytes_stream()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e));
            let reader = progress::wrap_reader(StreamReader::new(st), pb.clone());
            let mut reader = BufReader::new(reader);

            let dest = match output {
                Some(Destination::File(file)) => file,
                Some(Destination::Dir(dir)) => dir.join(filename.to_string()),
                None => PathBuf::from(filename.to_string()),
            };

            println!("Saving file to {}", dest.display());
            let out = File::create(dest).await?;
            let mut writer = BufWriter::new(out);

            if let Some(decrypt_key) = decrypt_key {
                decrypt::decrypt(&decrypt_key, &mut reader, &mut writer).await?;
            } else {
                tokio::io::copy(&mut reader, &mut writer).await?;
            }

            pb.finish_with_message("Download complete");
        }
        Some(("decrypt", matches)) => {
            let model = matches.value_of("model").expect("arg is required");
            let region = matches.value_of("region").expect("arg is required");
            let version = matches.value_of("version").expect("arg is required");

            let input = matches
                .value_of_os("input")
                .map(PathBuf::from)
                .expect("arg is required");

            let output = match matches.value_of_os("output").map(PathBuf::from) {
                Some(output) if output.is_dir() => Some(Destination::Dir(output)),
                Some(output) if !output.exists() => Some(Destination::File(output)),
                Some(output) if output.exists() => {
                    println!("Output file {} already exists", output.display());
                    return Ok(());
                }
                Some(_) | None => None,
            };

            let client = Client::new()?;
            let mut session = client.begin_session().await?;
            let info = client
                .file_info(&model, &region, &version, &mut session)
                .await?;

            print_info(&model, &region, &info);

            let (filename, decrypt_key) = match (
                info.decrypt_key,
                info.binary_name.strip_suffix(".enc4"),
                info.binary_name.strip_suffix(".enc2"),
            ) {
                (DecryptKey::V2(key), None, Some(filename))
                | (DecryptKey::V4(key), Some(filename), None) => {
                    (PathBuf::from(filename), key.to_vec())
                }
                (DecryptKey::Unknown, None, None) => {
                    println!("couldn't determine decryption key.");
                    return Ok(());
                }
                _ => unreachable!(),
            };

            let dest = match output {
                Some(Destination::File(file)) => file,
                Some(Destination::Dir(dir)) => dir.join(filename),
                None => filename,
            };

            println!("Decrypting file to {}", dest.display());
            let file = File::open(input).await?;

            let md = file.metadata().await?;
            let pb = progress::new(md.len());
            let file = progress::wrap_reader(file, pb);

            let mut reader = BufReader::new(file);

            let out = File::create(dest).await?;
            let mut writer = BufWriter::new(out);

            decrypt::decrypt(&decrypt_key, &mut reader, &mut writer).await?;
        }
        _ => {}
    }

    Ok(())
}

enum Destination {
    Dir(PathBuf),
    File(PathBuf),
}

fn print_info(model: &str, region: &str, info: &BinaryInfo) {
    println!("Name: {}", info.display_name);
    println!("Model: {}", model);
    println!("Region: {}", region);
    println!("Latest Version:");
    println!("  Version: {}", info.version);
    println!("  OS: {}", info.os_version);
    println!("  Filename: {}", info.binary_name);
    println!("  Size: {} bytes", info.binary_size);
    match info.decrypt_key {
        DecryptKey::V2(key) | DecryptKey::V4(key) => println!("  Decrypt key: {:02X}", key),
        DecryptKey::Unknown => println!("  Decrypt key is unknown"),
    }
}
