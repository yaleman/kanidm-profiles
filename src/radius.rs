use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct RadiusGroup {
    spn: String,
    vlan: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RadiusClient {
    name: String,
    ipaddr: String,
    secret: String,
}
