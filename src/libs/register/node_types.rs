use serde::{Deserialize, Serialize};

pub const TYPES_NODE_TYPE_LOOKUP_NODE: &str = "lookup";

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ServiceNode {
    pub uid : String,

    #[serde(rename="type")]
    pub node_type : String,

    #[serde(rename="api-root")]
    pub api_root : String,

    #[serde(rename="scheme", default)]
    pub scheme : String,
}

impl ServiceNode {
    pub fn new() -> Self {
        Default::default()
    }
    #[allow(dead_code)]
    fn valid(self) -> bool {
        !self.uid.is_empty() && !self.node_type.is_empty() && !self.api_root.is_empty()
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Default, Debug, Clone, Serialize, Deserialize)]
pub struct RegisterNode {
    pub uid : String,

    #[serde(rename="type")]
    pub node_type : String,

    #[serde(rename="api-root")]
    pub api_root : String,

    #[serde(rename="served-lookup-uid")]
    service_lookup_uid : String,

    // using String instead DateTime Format
    // because in RUST serde only support with annotation 'ts_second' in seconds

    #[serde(rename="create-time")]
    create_time : String,
}

impl RegisterNode {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Default::default()
    }
    #[allow(dead_code)]
    pub fn valid(self) -> bool {
        !self.uid.is_empty() && !self.node_type.is_empty() && !self.api_root.is_empty()
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct HttpRegisterNodes {
    #[serde(default)]
    pub nodes : Vec<RegisterNode>,
}

#[allow(dead_code)]
type HttpServiceQueryResponse = HttpRegisterNodes;

impl HttpRegisterNodes {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct NodeQueryFilter {
    pub exclude : Vec<String>,
    pub include : Vec<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct HttpServiceQueryRequest {
    #[serde(rename="from-uid")]
    pub from_uid : String,

    #[serde(rename="uid-filter")]
    pub uid_filter : NodeQueryFilter,

    #[serde(rename="type-filter")]
    pub type_filter : NodeQueryFilter,
}

impl HttpServiceQueryRequest {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct HttpPingRequest {
    #[serde(rename="from-node-type")]
    pub from_node_type : String,

    #[serde(rename="from-uid")]
    pub from_uid : String,

    #[serde(rename="to-uid")]
    pub to_uid : String,
}

impl HttpPingRequest {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct HttpPingResponse {
    #[serde(rename="from-uid")]
    pub from_uid : String,
}

impl HttpPingResponse {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct HttpApiRoot {
    #[serde(rename="api-root")]
    pub api_root : String,
}

impl HttpApiRoot {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct HttpServiceEventRequest {
    #[serde(rename="from-uid")]
    from_uid : String,

    #[serde(rename="event-id")]
    pub event_id : String,

    #[serde(rename="event-args",default)]
    pub event_args : Vec<String>,
}

impl HttpServiceEventRequest {
    pub fn new() -> Self {
        Default::default()
    }
}
