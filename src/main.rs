use colink_policy_module::colink_policy_module_proto::*;
use colink_sdk_a::*;
use colink_sdk_p::ProtocolEntry;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

struct PolicyModule {
    cl: CoLink,
    rules: Mutex<Vec<Rule>>,
}

impl PolicyModule {
    async fn _rule_monitor(&self) -> Result<(), Box<dyn std::error::Error>> {
        let queue_name = self
            .cl
            .subscribe("_colink_policy_module:settings", None)
            .await?;
        let mut subscriber = self.cl.new_subscriber(&queue_name).await?;
        loop {
            let data = subscriber.get_next().await?;
            debug!("Received [{}]", String::from_utf8_lossy(&data));
            let message: SubscriptionMessage = prost::Message::decode(&*data)?;
            if message.change_type != "delete" {
                let mut settings: Settings = prost::Message::decode(&*message.payload)?;
                if settings.enable {
                    let mut rules = self.rules.lock().await;
                    rules.clear();
                    rules.append(&mut settings.rules);
                    drop(rules);
                } else {
                    self.cl.unsubscribe(&queue_name).await?;
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

    async fn _operator(&self, queue_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut subscriber = self.cl.new_subscriber(queue_name).await?;
        loop {
            let data = subscriber.get_next().await?;
            debug!("Received [{}]", String::from_utf8_lossy(&data));
            let message: SubscriptionMessage = prost::Message::decode(&*data)?;
            if message.change_type != "delete" {
                let task_id: Task = prost::Message::decode(&*message.payload).unwrap();
                let res = self
                    .cl
                    .read_entries(&[StorageEntry {
                        key_name: format!("_colink_internal:tasks:{}", task_id.task_id),
                        ..Default::default()
                    }])
                    .await?;
                let task_entry = &res[0];
                let task: Task = prost::Message::decode(&*task_entry.payload).unwrap();
                if task.status == "waiting" {
                    // TODO match rules
                }
            }
        }
    }

    async fn operator(&self, queue_name: &str) -> Result<(), String> {
        match self._operator(queue_name).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

struct PolicyModuleLauncher;
#[colink_sdk_p::async_trait]
impl ProtocolEntry for PolicyModuleLauncher {
    async fn start(
        &self,
        cl: CoLink,
        _param: Vec<u8>,
        _participants: Vec<Participant>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pm = Arc::new(PolicyModule {
            cl: cl.clone(),
            rules: Mutex::new(Vec::new()),
        });
        let res = cl
            .read_entries(&[StorageEntry {
                key_name: "_colink_policy_module:settings".to_string(),
                ..Default::default()
            }])
            .await?;
        let mut settings: Settings = prost::Message::decode(&*res[0].payload)?;
        if settings.enable {
            let mut rules = pm.rules.lock().await;
            rules.append(&mut settings.rules);
            drop(rules);
            let rule_monitor = {
                let pm = pm.clone();
                tokio::spawn(async move { pm.rule_monitor().await })
            };
            let task_queue_name = cl.subscribe("_colink_internal:TODO", None).await?;
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

colink_sdk_p::protocol_start!("policy_module", ("launcher", PolicyModuleLauncher));
