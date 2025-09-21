use crate::handlers::common::{get_opts, get_queue_items};
use custom_logger as log;
use serde_derive::{Deserialize, Serialize};
use surrealkv::Durability;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormData {
    pub key: Option<String>,
    pub title: String,
    pub file: String,
    pub category: String,
    pub prompt: String,
    pub credentials: String,
    pub run_once: String,
    pub db: String,
}

pub trait QueueInterface {
    async fn process_queue() -> Result<(), Box<dyn std::error::Error>>;
}

pub struct Queue {}

impl QueueInterface for Queue {
    async fn process_queue() -> Result<(), Box<dyn std::error::Error>> {
        //first check to see if there are jobs on the queue
        let hm_queue = get_queue_items("queue".to_string()).await?;
        let key = hm_queue.keys();
        if key.len() > 0 {
            let s_key = str::from_utf8(key.last().unwrap())?;
            log::info!(
                "[process_queue] there is still an item in the queue with key {}",
                s_key
            );
            return Ok(());
        }

        // copy to queue
        let mut vec_key = vec![];
        let hm_formdata = get_queue_items("formdata".to_string()).await?;
        let tree = get_opts("queue".to_string())?;
        let mut txn = tree.begin()?;
        for (k, v) in hm_formdata.iter() {
            let json_data = serde_json::to_string(&v)?;
            let key = k.clone();
            txn.set(&key, json_data.as_bytes())?;
            vec_key.push(key.clone());
        }
        txn.commit().await?;
        tree.close().await?;

        // try and delete
        if vec_key.len() > 0 {
            let tree_from = get_opts("formdata".to_string())?;
            let mut txn_from = tree_from.begin()?;
            txn_from.set_durability(Durability::Immediate);
            let s_key = str::from_utf8(&vec_key[0])?;
            log::info!("[process_queue] deleting key {}", s_key);
            txn_from.delete(&vec_key[0])?;
            txn_from.commit().await?;
            tree_from.close().await?;
        }

        Ok(())
    }
}
