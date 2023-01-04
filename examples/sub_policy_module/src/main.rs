#![allow(unused_variables)]
use colink::*;

struct Init;
#[colink::async_trait]
impl ProtocolEntry for Init {
    async fn start(
        &self,
        cl: CoLink,
        _param: Vec<u8>,                 // For init function, param is empty
        _participants: Vec<Participant>, // For init function, participants is empty
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let task_queue_name = cl.subscribe("_policy_module:sub:greetings", None).await?;
        cl.update_entry("greetings:policy_module_queue", task_queue_name.as_bytes())
            .await?;
        let participants = vec![Participant {
            user_id: cl.get_user_id()?,
            role: "default".to_string(),
        }];
        cl.run_task(
            "greetings.policy_module",
            Default::default(),
            &participants,
            false,
        )
        .await?;
        Ok(())
    }
}

struct PM;
#[colink::async_trait]
impl ProtocolEntry for PM {
    async fn start(
        &self,
        cl: CoLink,
        _param: Vec<u8>,
        _participants: Vec<Participant>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let queue_name =
            String::from_utf8_lossy(&cl.read_or_wait("greetings:policy_module_queue").await?)
                .to_string();
        let mut subscriber = cl.new_subscriber(&queue_name).await?;
        loop {
            let data = subscriber.get_next().await?;
            let message: SubscriptionMessage = prost::Message::decode(&*data)?;
            if message.change_type != "delete" {
                let task_id: Task = prost::Message::decode(&*message.payload).unwrap();
                let res = cl
                    .read_entry(&format!("_internal:tasks:{}", task_id.task_id))
                    .await?;
                let task: Task = prost::Message::decode(&*res).unwrap();
                if task.status == "waiting" {
                    let msg = String::from_utf8_lossy(&task.protocol_param);
                    if msg == "hello" {
                        // auto apporve "hello" message
                        cl.confirm_task(&task.task_id, true, false, "").await?;
                    }
                }
            }
        }
    }
}

struct Initiator;
#[colink::async_trait]
impl ProtocolEntry for Initiator {
    async fn start(
        &self,
        cl: CoLink,
        param: Vec<u8>,
        participants: Vec<Participant>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        println!("initiator");
        Ok(())
    }
}

struct Receiver;
#[colink::async_trait]
impl ProtocolEntry for Receiver {
    async fn start(
        &self,
        cl: CoLink,
        param: Vec<u8>,
        participants: Vec<Participant>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        println!("{}", String::from_utf8_lossy(&param));
        cl.create_entry(&format!("tasks:{}:output", cl.get_task_id()?), &param)
            .await?;
        Ok(())
    }
}

colink::protocol_start!(
    ("greetings:@init", Init),               // bind init function
    ("greetings.policy_module:default", PM), // bind policy module
    ("greetings:initiator", Initiator),      // bind initiator's entry function
    ("greetings:receiver", Receiver)         // bind receiver's entry function
);
