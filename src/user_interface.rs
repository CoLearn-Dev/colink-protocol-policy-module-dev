use crate::colink_policy_module_proto::*;
use colink_sdk_a::*;
use colink_sdk_p::ProtocolEntry;
use prost::Message;
use tracing::error;
pub struct UserStart;
#[colink_sdk_p::async_trait]
impl ProtocolEntry for UserStart {
    async fn start(
        &self,
        cl: CoLink,
        _param: Vec<u8>,
        _participants: Vec<Participant>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut settings: Settings = match cl.read_entry("_policy_module:settings").await {
            Ok(res) => prost::Message::decode(&*res)?,
            Err(_) => Default::default(),
        };
        if settings.enable {
            Err("The policy module has already been started.")?
        }
        settings.enable = true;
        let mut payload = vec![];
        settings.encode(&mut payload).unwrap();
        cl.update_entry("_policy_module:settings", &payload).await?;
        let participants = vec![Participant {
            user_id: cl.get_user_id()?,
            ptype: "local".to_string(),
        }];
        cl.run_task("policy_module", Default::default(), &participants, false)
            .await?;
        Ok(())
    }
}

pub struct UserStop;
#[colink_sdk_p::async_trait]
impl ProtocolEntry for UserStop {
    async fn start(
        &self,
        cl: CoLink,
        _param: Vec<u8>,
        _participants: Vec<Participant>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut settings: Settings = match cl.read_entry("_policy_module:settings").await {
            Ok(res) => prost::Message::decode(&*res)?,
            Err(_) => Default::default(),
        };
        if !settings.enable {
            Err("The policy module is not running.")?
        }
        settings.enable = false;
        let mut payload = vec![];
        settings.encode(&mut payload).unwrap();
        cl.update_entry("_policy_module:settings", &payload).await?;
        Ok(())
    }
}

pub struct UserAddProtocol;
#[colink_sdk_p::async_trait]
impl ProtocolEntry for UserAddProtocol {
    async fn start(
        &self,
        cl: CoLink,
        param: Vec<u8>,
        _participants: Vec<Participant>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut settings: Settings = match cl.read_entry("_policy_module:settings").await {
            Ok(res) => prost::Message::decode(&*res)?,
            Err(_) => Default::default(),
        };
        let rule = Rule {
            task_filter: Some(TaskFilter {
                protocol_name: String::from_utf8_lossy(&param).to_string(),
                ..Default::default()
            }),
            action: "approve".to_string(),
            priority: 1,
        };
        settings.rules.push(rule);
        let mut payload = vec![];
        settings.encode(&mut payload).unwrap();
        cl.update_entry("_policy_module:settings", &payload).await?;
        Ok(())
    }
}

pub struct UserRemoveProtocol;
#[colink_sdk_p::async_trait]
impl ProtocolEntry for UserRemoveProtocol {
    async fn start(
        &self,
        cl: CoLink,
        param: Vec<u8>,
        _participants: Vec<Participant>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut settings: Settings = match cl.read_entry("_policy_module:settings").await {
            Ok(res) => prost::Message::decode(&*res)?,
            Err(_) => Default::default(),
        };
        let protocol_name = String::from_utf8_lossy(&param).to_string();
        let mut index = usize::MAX;
        for i in 0..settings.rules.len() {
            if settings.rules[i]
                .task_filter
                .as_ref()
                .unwrap()
                .protocol_name
                == protocol_name
            {
                index = i;
            }
        }
        if index != usize::MAX {
            settings.rules.remove(index);
            let mut payload = vec![];
            settings.encode(&mut payload).unwrap();
            cl.update_entry("_policy_module:settings", &payload).await?;
        } else {
            error!("Rule not found.");
        }
        Ok(())
    }
}
