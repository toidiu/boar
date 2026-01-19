use crate::{EndpointSetup, NetworkSetup};
use byte_unit::Byte;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutionPlan {
    pub uuid: Uuid,
    pub network_setup: NetworkSetup,
    pub endpoint_setup: EndpointSetup,
    pub download_bytes: Byte,
    pub count: u16,
}
