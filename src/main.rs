//! Quick config switcher for kanidm CLI profiles

use std::fs::File;
use std::io::Write;

use clap::{Parser, Subcommand};
use console::{style, Term};
use kanidm_profiles::client::KanidmClientConfig;
use kanidm_profiles::profiles::KanidmProfiles;
use kanidm_profiles::CONFIG_PATH;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Profile to act on
    #[arg(short, long)]
    profile: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    Get {
        /// Field to get, use JSON path references like '/radius_clients/0/secret'
        #[arg()]
        field: String,
    },
    Set {
        /// Field to set, use JSON path references like '/radius_clients/0/secret'
        #[arg()]
        field: String,
        #[arg()]
        value: String,
    },
}

/// Handles the get command
fn get_field_value(field: String, profile: &KanidmClientConfig) {
    eprintln!(
        "You asked to get field {} from {}",
        field,
        profile.name.clone()
    );
    let parsed_profile = serde_json::json!(profile);
    match parsed_profile.pointer(&field) {
        Some(val) => println!("{}", val.as_str().unwrap()),
        None => {
            eprintln!("Couldn't find {field:?}");
            std::process::exit(1);
        }
    }
}
/// Handles the set command
fn set_field_value(field: String, value: String, profile: &KanidmClientConfig) {
    eprintln!(
        "You asked to set field {} from {}",
        field,
        profile.name.clone()
    );
    let mut parsed_profile = serde_json::json!(profile);
    match parsed_profile.pointer_mut(&field) {
        Some(val) => {
            println!("{}", val.as_str().unwrap());
            let newval = serde_json::Value::from(value);
            *val = newval;
        }
        None => {
            eprintln!("Couldn't find {field:?}");
            std::process::exit(1);
        }
    }
    eprintln!("Profile:\n{:#?}", parsed_profile);
}

fn main() {
    let profiles = match KanidmProfiles::default().parse_config_profiles() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("Failed to load config: {:?}", error);
            std::process::exit(1);
        }
    };

    let cli = Cli::parse();

    if profiles.profiles.is_empty() {
        println!("No profiles specified, please create some in ~/.config/kanidm-profiles.toml");
        std::process::exit(0);
    }

    let profile: Option<usize> = match cli.profile.clone() {
        None => {
            match dialoguer::Select::new()
                .items(&profiles.profiles)
                .default(0)
                .report(true)
                .with_prompt("Please select a new profile:")
                .interact_on_opt(&Term::stderr())
            {
                Ok(value) => value,
                Err(error) => {
                    eprintln!("Uh...? {:?}", error);
                    std::process::exit(1);
                }
            }
        }
        Some(profile_name) => {
            profiles
                .profiles
                .iter()
                .enumerate()
                .find_map(|(index, profile)| {
                    match profile.name == profile_name {
                        true => Some(index),
                        false => {
                            // println!("{} didn't match {}", profile.name, profile_name);
                            None
                        }
                    }
                })
        }
    };

    let profile = match profile {
        Some(val) => profiles.profiles.get(val).unwrap(),
        None => {
            eprintln!("Couldn't find matching profile, quitting!");
            std::process::exit(1);
        }
    };

    if let Some(command) = cli.command {
        match command {
            Commands::Get { field } => {
                get_field_value(field, profile);
            }
            Commands::Set { field, value } => {
                set_field_value(field, value, profile);
            }
        };
        std::process::exit(0);
    }

    println!("{}", style("Applying new config").green());

    println!("{}", toml::to_string(profile).unwrap());

    let config_path: String = shellexpand::tilde(CONFIG_PATH).into();
    let mut file = File::create(config_path).unwrap();

    match file.write(toml::to_string(profile).unwrap().as_bytes()) {
        Ok(woot) => println!(
            "{}",
            style(format!(
                "Successfully wrote {:?} bytes to the new config file.",
                woot
            ))
            .green()
        ),
        Err(error) => println!("{}", style(format!("Oh no: {:?}", error)).red()),
    }
}
