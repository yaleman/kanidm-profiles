//! Quick config switcher for kanidm CLI profiles

use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::path::Path;

use clap::{Parser, Subcommand};
use console::{style, Term};
use serde::{Deserialize, Serialize};
const CONFIG_PATH: &str = "~/.config/kanidm";
const PROFILE_PATH: &str = "~/.config/kanidm-profiles.toml";

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

#[derive(Debug, Deserialize, Serialize)]
struct RadiusGroup {
    spn: String,
    vlan: u16,
}

#[derive(Debug, Deserialize, Serialize)]
struct RadiusClient {
    name: String,
    ipaddr: String,
    secret: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct KanidmClientConfig {
    #[serde(flatten)]
    kanidm_client_config: kanidm_client::KanidmClientConfig,

    #[serde(skip_serializing)]
    name: String,
    #[allow(dead_code)]
    username: Option<String>,
    #[allow(dead_code)]
    password: Option<String>,
    // Used in the RADIUS integration
    #[allow(dead_code)]
    radius_required_groups: Option<Vec<String>>,
    // Used in the RADIUS integration
    #[allow(dead_code)]
    radius_groups: Option<Vec<RadiusGroup>>,
    // Used in the RADIUS integration
    #[allow(dead_code)]
    radius_clients: Option<Vec<RadiusClient>>,
}

impl std::fmt::Display for KanidmClientConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        let uri = match &self.kanidm_client_config.uri {
            Some(value) => value,
            None => "<unset>",
        };

        f.write_str(&format!("{} ({})", self.name.as_str(), &uri))
    }
}

#[derive(Debug, Default, Deserialize)]
struct KanidmProfiles {
    profiles: Vec<KanidmClientConfig>,
}

impl KanidmProfiles {
    /// let's parse the thing
    fn parse_config_profiles(self) -> Result<Self, ()> {
        eprintln!("Attempting to load profiles from {:#?}", &PROFILE_PATH);
        // If the file does not exist, we skip this function.
        let loadpath: String = shellexpand::tilde(&PROFILE_PATH).into();

        let testpath = Path::new(loadpath.as_str());
        if !testpath.exists() {
            eprintln!("Failed to find config file: {testpath:?}");
            return Err(());
        }
        let mut f = match File::open(loadpath) {
            Ok(f) => {
                eprintln!(
                    "Successfully opened configuration file {:#?}",
                    &PROFILE_PATH
                );
                f
            }
            Err(e) => {
                match e.kind() {
                    ErrorKind::NotFound => {
                        eprintln!(
                            "Configuration file {:#?} not found, please create one!",
                            &PROFILE_PATH
                        );
                    }
                    ErrorKind::PermissionDenied => {
                        eprintln!(
                            "Permission denied loading configuration file {:#?}, bailing.",
                            &PROFILE_PATH
                        );
                    }
                    _ => {
                        eprintln!(
                            "Unable to open config file {:#?} [{:?}], bailing ...",
                            &PROFILE_PATH, e
                        );
                    }
                };
                std::process::exit(1);
            }
        };

        let mut contents = String::new();
        f.read_to_string(&mut contents)
            .map_err(|e| eprintln!("Failed to parse kanidm-profiles config file: {:?}", e))?;

        let config: KanidmProfiles =
            toml::from_str(contents.as_str()).map_err(|e| eprintln!("{:?}", e))?;
        Ok(config)
    }
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
