//! Quick config switcher for kanidm CLI profiles

use std::fs::File;

use serde::{Deserialize, Serialize};
use std::io::{ErrorKind, Read, Write};

use console::{style, Term};
const CONFIG_PATH: &str = "~/.config/kanidm";
const PROFILE_PATH: &str = "~/.config/kanidm-profiles.toml";

#[derive(Debug, Deserialize, Serialize)]
struct RadiusGroup {
    name: String,
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
    name: String,
    uri: String,
    verify_ca: Option<bool>,
    verify_hostnames: Option<bool>,
    ca_path: Option<String>,
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
        f.write_str(format!("{} ({})", self.name.as_str(), self.uri.as_str()).as_str())
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
            .map_err(|e| eprintln!("{:?}", e))?;

        let config: KanidmProfiles =
            toml::from_str(contents.as_str()).map_err(|e| eprintln!("{:?}", e))?;
        Ok(config)
    }
}

fn main() {
    let profiles = match KanidmProfiles::default().parse_config_profiles() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("Failed to load config: {:?}", error);
            std::process::exit(1);
        }
    };

    if profiles.profiles.is_empty() {
        println!("No profiles specified, please create some in ~/.config/kanidm-profiles.toml");
        std::process::exit(0);
    }

    let selected_profile = match dialoguer::Select::new()
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
    };

    let profile = profiles.profiles.get(selected_profile.unwrap()).unwrap();
    println!("{}", style("Applying new config").green());

    println!("{}", toml::to_string(profile).unwrap());

    let config_path: String = shellexpand::tilde(CONFIG_PATH).into();
    let mut file = File::create(config_path).unwrap();

    match file.write(toml::to_string(profile).unwrap().as_bytes()) {
        Ok(woot) => println!(
            "{}",
            style(format!("Successfully wrote {:?} bytes to the new config file.",
            woot)).green()
        ),
        Err(error) => println!(
            "{}",
            style(format!("Oh no: {:?}", error)).red()),
    }
}
