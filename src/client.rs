use serde::{Deserialize, Serialize};

use crate::radius::{RadiusClient, RadiusGroup};

#[derive(Debug, Deserialize, Serialize)]
pub struct KanidmClientConfig {
    #[serde(flatten)]
    kanidm_client_config: kanidm_client::KanidmClientConfig,

    #[serde(skip_serializing)]
    pub name: String,
    #[allow(dead_code)]
    pub username: Option<String>,
    #[allow(dead_code)]
    pub password: Option<String>,
    // Used in the RADIUS integration
    #[allow(dead_code)]
    pub radius_required_groups: Option<Vec<String>>,
    // Used in the RADIUS integration
    #[allow(dead_code)]
    pub radius_groups: Option<Vec<RadiusGroup>>,
    // Used in the RADIUS integration
    #[allow(dead_code)]
    pub radius_clients: Option<Vec<RadiusClient>>,
}

impl std::fmt::Display for KanidmClientConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        let default_uri = match &self.kanidm_client_config.default.uri {
            Some(value) => value.as_str(),
            None => "<unset>",
        };

        f.write_str(&format!("{} ({})", self.name.as_str(), &default_uri))?;

        if !self.kanidm_client_config.instances.is_empty() {
            f.write_str("sub-instances:")?;
            for (name, config) in self.kanidm_client_config.instances.iter() {
                let uri = match config.uri.as_ref() {
                    Some(value) => value,
                    None => &"<unset>".to_string(),
                };
                f.write_str(&format!(" - instance uri: {} ({})", name, &uri))?;
            }
        }
        Ok(())
    }
}
