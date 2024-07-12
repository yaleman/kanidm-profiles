use std::fs::File;
use std::path::Path;

use serde::Deserialize;
use std::io::{ErrorKind, Read};

use crate::client::KanidmClientConfig;
use crate::PROFILE_PATH;

#[derive(Debug, Default, Deserialize)]
pub struct KanidmProfiles {
    pub profiles: Vec<KanidmClientConfig>,
}

impl KanidmProfiles {
    /// let's parse the thing
    pub fn parse_config_profiles(self) -> Result<Self, String> {
        eprintln!("Attempting to load profiles from {:#?}", &PROFILE_PATH);
        // If the file does not exist, we skip this function.
        let loadpath: String = shellexpand::tilde(&PROFILE_PATH).into();

        let testpath = Path::new(loadpath.as_str());
        if !testpath.exists() {
            eprintln!("Failed to find config file: {testpath:?}");
            return Err(format!("Failed to find config file: {testpath:?}"));
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
            .map_err(|e| format!("Failed to parse kanidm-profiles config file: {:?}", e))?;

        let config: KanidmProfiles =
            toml::from_str(contents.as_str()).map_err(|e| format!("{:?}", e))?;
        Ok(config)
    }
}
