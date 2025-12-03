mod api;
mod model;

use clap::{App, Arg};
use keyring::Entry;
use std::path::Path;

static CLIENT_ID: &str = "Y5NOImSXBO6vCmiVN7hmFgSe4WKo71hO";

fn store_token(entry: &Entry, client: &api::Client) {
    entry
        .set_password(serde_json::to_string(&client.token).unwrap().as_ref())
        .expect("Failed to save new token");
}

fn main() {
    let m = App::new("yoto-cli")
        .author("Louis-Francis Ratté-Boulianne, louis-francis@ratte-boulianne.com")
        .version("0.1.0")
        .about("Tool to create and manage Yoto cards and devices")
        .subcommand(App::new("login"))
        .subcommand(App::new("logout"))
        .subcommand(App::new("devices"))
        .subcommand(
            App::new("card")
                .subcommand(App::new("list"))
                .subcommand(App::new("info").arg(Arg::with_name("id").index(1)))
                .subcommand(
                    App::new("backup").arg(Arg::with_name("id").index(1)).arg(
                        Arg::with_name("path")
                            .long("path")
                            .takes_value(true)
                            .help("Path where to create the backup directory"),
                    ),
                ),
        )
        .subcommand(App::new("upload").arg(Arg::with_name("path").index(1)))
        .get_matches();

    let entry = Entry::new("yoto-api", "oauth").unwrap();
    let token = match entry.get_password() {
        Ok(password) => serde_json::from_str(&password).ok(),
        Err(_) => None,
    };
    let mut client = api::Client::new(CLIENT_ID, token);

    match m.subcommand() {
        Some(("login", _)) => {
            match client.auth() {
                Ok(()) => store_token(&entry, &client),
                Err(_) => println!("ERROR: Failed to login"),
            }
            return;
        }
        Some(("logout", _)) => {
            let _ = entry.delete_credential();
            return;
        }
        _ => (),
    }

    /* Other commands need authentication */
    if client.token.is_none() {
        println!("Please authenticate before using other commands");
        return;
    }
    match client.refresh_token() {
        api::RefreshStatus::AlreadyValid => (),
        api::RefreshStatus::Refreshed => store_token(&entry, &client),
        api::RefreshStatus::Failed => {
            println!("Failed to refresh expired authentication token");
            return;
        }
    };

    match m.subcommand() {
        Some(("devices", _)) => {
            let devices = client.get_devices();
            if devices.is_empty() {
                println!("No devices linked with this account.");
            } else {
                println!("Devices:");
                for device in devices.iter() {
                    println!("  - {} ({})", device.name, device.id);
                }
            }
        }
        Some(("card", command)) => match command.subcommand() {
            Some(("list", arg)) => {
                let cards = client.get_cards();
                if cards.is_empty() {
                    println!("No cards linked to this account.");
                } else {
                    println!("Cards:");
                    for card in cards.iter() {
                        println!("   {}:  {}", card.card_id, card.title);
                    }
                }
            }
            Some(("info", arg)) => {
                if let Some(id) = arg.value_of("id") {
                    match client.get_card(id, false) {
                        Ok(card) => {
                            println!("Card {}:", id);
                            println!("{:?}", card);
                        }
                        Err(_) => {
                            println!("Error while retrieving details for card \"{}\"", id);
                        }
                    }
                }
            }
            Some(("backup", arg)) => {
                if let Some(id) = arg.value_of("id") {
                    match client.get_card(id, false) {
                        Ok(card) => {
                            println!("Card {}:", id);
                            println!("{:?}", card);
                        }
                        Err(_) => {
                            println!("Error while retrieving details for card \"{}\"", id);
                        }
                    }
                }
            }
            _ => {
                println!("Invalid card command");
                return;
            }
        },
        Some(("upload", arg)) => {
            if let Some(path) = arg.value_of("path") {
                let uuid = client.upload_audio_file(Path::new(path)).unwrap();
                println!("Upload SHA256: {}", uuid);
            }
        }
        _ => (),
    }
}
