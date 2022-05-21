use crate::colink_policy_module_proto::*;
use colink_sdk_a::*;
use colink_sdk_p::ProtocolEntry;
use regex::Regex;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, warn};

pub struct UserStart;
#[colink_sdk_p::async_trait]
impl ProtocolEntry for UserStart {
    async fn start(
        &self,
        cl: CoLink,
        _param: Vec<u8>,
        _participants: Vec<Participant>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // self.read_entries()
        Ok(())
    }
}
