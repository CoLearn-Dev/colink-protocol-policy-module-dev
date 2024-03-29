use crate::colink_policy_module_proto::*;
use colink::*;
use regex::Regex;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, warn};

struct PolicyModule {
    cl: CoLink,
    rules: Mutex<Vec<Rule>>,
}

impl PolicyModule {
    async fn _rule_monitor(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let queue_name = self.cl.subscribe("_policy_module:settings", None).await?;
        let storage_entry = StorageEntry {
            key_name: "_policy_module:settings".to_string(),
            ..Default::default()
        };
        let res = self.cl.read_entries(&[storage_entry]).await?;
        let rule_timestamp = get_timestamp(&res[0].key_path);
        let mut settings: Settings = prost::Message::decode(&*res[0].payload)?;
        let mut rules = self.rules.lock().await;
        rules.clear();
        rules.append(&mut settings.rules);
        drop(rules);
        self.cl
            .update_entry(
                "_policy_module:applied_settings_timestamp",
                &rule_timestamp.to_le_bytes(),
            )
            .await?;
        let mut subscriber = self.cl.new_subscriber(&queue_name).await?;
        loop {
            let data = subscriber.get_next().await?;
            debug!("Received [{}]", String::from_utf8_lossy(&data));
            let message: SubscriptionMessage = prost::Message::decode(&*data)?;
            let rule_timestamp = get_timestamp(&message.key_path);
            if message.change_type != "delete" {
                let mut settings: Settings = prost::Message::decode(&*message.payload)?;
                if settings.enable {
                    settings.rules.sort_by(|a, b| a.priority.cmp(&b.priority));
                    let mut rules = self.rules.lock().await;
                    rules.clear();
                    rules.append(&mut settings.rules);
                    drop(rules);
                    self.cl
                        .update_entry(
                            "_policy_module:applied_settings_timestamp",
                            &rule_timestamp.to_le_bytes(),
                        )
                        .await?;
                } else {
                    self.cl.unsubscribe(&queue_name).await?;
                    self.cl
                        .update_entry(
                            "_policy_module:applied_settings_timestamp",
                            &rule_timestamp.to_le_bytes(),
                        )
                        .await?;
                    return Ok(());
                }
            }
        }
    }

    async fn rule_monitor(&self) -> Result<(), String> {
        match self._rule_monitor().await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn _operator(
        &self,
        queue_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let mut subscriber = self.cl.new_subscriber(queue_name).await?;
        loop {
            let data = subscriber.get_next().await?;
            debug!("Received [{}]", String::from_utf8_lossy(&data));
            let message: SubscriptionMessage = prost::Message::decode(&*data)?;
            if message.change_type != "delete" {
                let task_id: Task = prost::Message::decode(&*message.payload).unwrap();
                let res = self
                    .cl
                    .read_entry(&format!("_internal:tasks:{}", task_id.task_id))
                    .await?;
                let task: Task = prost::Message::decode(&*res).unwrap();
                if task.status == "waiting" {
                    let rules = self.rules.lock().await.clone();
                    let mut matched_priority = i64::MAX;
                    let mut matched_action = Action::default();
                    for rule in rules {
                        if rule.priority as i64 > matched_priority {
                            break;
                        }
                        if rule.task_filter.is_some()
                            && self.match_filter(&task, &rule.task_filter.unwrap())
                        {
                            if matched_priority == i64::MAX {
                                matched_priority = rule.priority as i64;
                                matched_action = rule.action.unwrap();
                            } else if matched_action != rule.action.unwrap() {
                                matched_priority = -1;
                                break;
                            }
                        }
                    }
                    if matched_priority == -1 {
                        warn!("rules conflict when matching task {}", task.task_id);
                    } else if matched_priority != i64::MAX {
                        let cl = self.cl.clone();
                        tokio::spawn(async move {
                            if matched_action.r#type == "approve" {
                                cl.confirm_task(&task.task_id, true, false, "").await?;
                            } else if matched_action.r#type == "reject" {
                                cl.confirm_task(&task.task_id, false, true, "").await?;
                            } else if matched_action.r#type == "ignore" {
                                cl.confirm_task(&task.task_id, false, false, "").await?;
                            } else if matched_action.r#type == "forward" {
                                cl.update_entry(
                                    &matched_action.forward_target_keyname,
                                    &message.payload,
                                )
                                .await?;
                            }
                            Ok::<(), Box<dyn std::error::Error + Send + Sync + 'static>>(())
                        });
                    }
                }
            }
        }
    }

    fn match_filter(&self, task: &Task, filter: &TaskFilter) -> bool {
        if filter.task_id != String::default() && filter.task_id != task.task_id {
            return false;
        }
        if filter.protocol_name != String::default() && filter.protocol_name != task.protocol_name {
            return false;
        }
        if filter.require_agreement.is_some()
            && filter.require_agreement.unwrap() != task.require_agreement
        {
            return false;
        }
        if !filter.participants.is_empty() {
            for p in &task.participants {
                if !filter.participants.contains(&p.user_id) {
                    return false;
                }
            }
        }
        if filter.role != String::default() {
            for p in &task.participants {
                if p.user_id == self.cl.get_user_id().unwrap() {
                    let regex = match Regex::new(&filter.role) {
                        Ok(regex) => regex,
                        Err(e) => {
                            warn!("invalid role: {}", e);
                            return false;
                        }
                    };
                    if !regex.is_match(&p.role) {
                        return false;
                    }
                    break;
                }
            }
        }
        if filter.parent_task_filter.is_some() {
            let parent_task_filter = filter.parent_task_filter.as_ref().unwrap();
            if parent_task_filter.parent_task_filter.is_none() {
                if !self.match_filter(task, parent_task_filter) {
                    return false;
                }
            } else {
                warn!("invalid parent_task_filter");
                return false;
            }
        }
        true
    }

    async fn operator(&self, queue_name: &str) -> Result<(), String> {
        match self._operator(queue_name).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub struct PolicyModuleLauncher;
#[colink::async_trait]
impl ProtocolEntry for PolicyModuleLauncher {
    async fn start(
        &self,
        cl: CoLink,
        _param: Vec<u8>,
        _participants: Vec<Participant>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let pm = Arc::new(PolicyModule {
            cl: cl.clone(),
            rules: Mutex::new(Vec::new()),
        });
        let storage_entry = StorageEntry {
            key_name: "_policy_module:settings".to_string(),
            ..Default::default()
        };
        let res = cl.read_entries(&[storage_entry]).await?;
        let rule_timestamp = get_timestamp(&res[0].key_path);
        let mut settings: Settings = prost::Message::decode(&*res[0].payload)?;
        if settings.enable {
            let mut rules = pm.rules.lock().await;
            rules.append(&mut settings.rules);
            drop(rules);
            let rule_monitor = {
                let pm = pm.clone();
                tokio::spawn(async move { pm.rule_monitor().await })
            };
            let task_queue_name =
                String::from_utf8_lossy(&cl.read_or_wait("_policy_module:task_queue_name").await?)
                    .to_string();
            cl.update_entry(
                "_policy_module:applied_settings_timestamp",
                &rule_timestamp.to_le_bytes(),
            )
            .await?;
            let operator = {
                let queue_name = task_queue_name.clone();
                let pm = pm.clone();
                tokio::spawn(async move { pm.operator(&queue_name).await })
            };
            rule_monitor.await??;
            operator.abort();
            cl.unsubscribe(&task_queue_name).await?;
        }
        Ok(())
    }
}

fn get_timestamp(key_path: &str) -> i64 {
    let pos = key_path.rfind('@').unwrap();
    key_path[pos + 1..].parse().unwrap()
}
