use crate::colink_policy_module_proto::*;
use colink::*;
use prost::Message;
use tracing::error;
pub struct UserStart;
#[colink::async_trait]
impl ProtocolEntry for UserStart {
    async fn start(
        &self,
        cl: CoLink,
        _param: Vec<u8>,
        _participants: Vec<Participant>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let lock = cl.lock("_policy_module:settings").await?;
        let mut settings: Settings = match cl.read_entry("_policy_module:settings").await {
            Ok(res) => prost::Message::decode(&*res)?,
            Err(_) => Default::default(),
        };
        if settings.enable {
            cl.unlock(lock).await?;
            return Err("The policy module has already been started.")?;
        }
        settings.enable = true;
        let mut payload = vec![];
        settings.encode(&mut payload).unwrap();
        cl.update_entry("_policy_module:settings", &payload).await?;
        cl.unlock(lock).await?;
        let participants = vec![Participant {
            user_id: cl.get_user_id()?,
            role: "local".to_string(),
        }];
        cl.run_task("policy_module", Default::default(), &participants, false)
            .await?;
        Ok(())
    }
}

pub struct UserStop;
#[colink::async_trait]
impl ProtocolEntry for UserStop {
    async fn start(
        &self,
        cl: CoLink,
        _param: Vec<u8>,
        _participants: Vec<Participant>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let lock = cl.lock("_policy_module:settings").await?;
        let mut settings: Settings = match cl.read_entry("_policy_module:settings").await {
            Ok(res) => prost::Message::decode(&*res)?,
            Err(_) => Default::default(),
        };
        if !settings.enable {
            cl.unlock(lock).await?;
            return Err("The policy module is not running.")?;
        }
        settings.enable = false;
        let mut payload = vec![];
        settings.encode(&mut payload).unwrap();
        cl.update_entry("_policy_module:settings", &payload).await?;
        cl.unlock(lock).await?;
        Ok(())
    }
}

pub struct UserAddProtocol;
#[colink::async_trait]
impl ProtocolEntry for UserAddProtocol {
    async fn start(
        &self,
        cl: CoLink,
        param: Vec<u8>,
        _participants: Vec<Participant>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let lock = cl.lock("_policy_module:settings").await?;
        let mut settings: Settings = match cl.read_entry("_policy_module:settings").await {
            Ok(res) => prost::Message::decode(&*res)?,
            Err(_) => Default::default(),
        };
        let rule_id = uuid::Uuid::new_v4().to_string();
        let rule = Rule {
            rule_id: rule_id.clone(),
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
        cl.unlock(lock).await?;
        cl.create_entry(
            &format!("tasks:{}:output", cl.get_task_id()?),
            rule_id.as_bytes(),
        )
        .await?;
        Ok(())
    }
}

pub struct UserRemoveRule;
#[colink::async_trait]
impl ProtocolEntry for UserRemoveRule {
    async fn start(
        &self,
        cl: CoLink,
        param: Vec<u8>,
        _participants: Vec<Participant>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let lock = cl.lock("_policy_module:settings").await?;
        let mut settings: Settings = match cl.read_entry("_policy_module:settings").await {
            Ok(res) => prost::Message::decode(&*res)?,
            Err(_) => Default::default(),
        };
        let rule_id = String::from_utf8_lossy(&param).to_string();
        let mut index = usize::MAX;
        for i in 0..settings.rules.len() {
            if settings.rules[i].rule_id == rule_id {
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
        cl.unlock(lock).await?;
        Ok(())
    }
}

pub struct UserReset;
#[colink::async_trait]
impl ProtocolEntry for UserReset {
    async fn start(
        &self,
        cl: CoLink,
        _param: Vec<u8>,
        _participants: Vec<Participant>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let lock = cl.lock("_policy_module:settings").await?;
        let mut settings: Settings = match cl.read_entry("_policy_module:settings").await {
            Ok(res) => prost::Message::decode(&*res)?,
            Err(_) => Default::default(),
        };
        settings.rules.clear();
        let mut payload = vec![];
        settings.encode(&mut payload).unwrap();
        cl.update_entry("_policy_module:settings", &payload).await?;
        cl.unlock(lock).await?;
        Ok(())
    }
}
