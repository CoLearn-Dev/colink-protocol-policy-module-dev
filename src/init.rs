use crate::colink_policy_module_proto::*;
use colink::*;
use prost::Message;

pub struct Init;
#[colink::async_trait]
impl ProtocolEntry for Init {
    async fn start(
        &self,
        cl: CoLink,
        _param: Vec<u8>,
        _participants: Vec<Participant>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let mut settings = Settings {
            enable: true,
            ..Default::default()
        };
        if let Ok(accept_all_tasks) = cl.read_entry("_policy_module:init:accept_all_tasks").await {
            let accept_all_tasks = String::from_utf8_lossy(&accept_all_tasks);
            if accept_all_tasks == "true" {
                let rule_id = uuid::Uuid::new_v4().to_string();
                let rule = Rule {
                    rule_id,
                    task_filter: Some(TaskFilter::default()),
                    action: Some(Action {
                        r#type: "approve".to_string(),
                        ..Default::default()
                    }),
                    priority: 1,
                };
                settings.rules.push(rule);
            }
        }
        let mut payload = vec![];
        settings.encode(&mut payload).unwrap();
        cl.update_entry("_policy_module:settings", &payload).await?;
        let task_queue_name = cl
            .subscribe("_internal:tasks:status:waiting:latest", None)
            .await?;
        cl.update_entry("_policy_module:task_queue_name", task_queue_name.as_bytes())
            .await?;
        let participants = vec![Participant {
            user_id: cl.get_user_id()?,
            role: "local".to_string(),
        }];
        cl.run_task("policy_module", Default::default(), &participants, false)
            .await?;
        Ok(())
    }
}
