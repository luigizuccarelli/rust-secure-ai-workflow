use crate::handlers::queue::FormData;
use crate::{error::handler::TaskExecuteError, MAP_LOOKUP};
use chrono::{DateTime, Duration, Utc};
use custom_logger as log;
use hyper::body::Bytes;
use std::collections::HashMap;
use surrealkv::{Durability, Tree, TreeBuilder};

pub fn get_box_error(msg: String) -> Box<dyn std::error::Error> {
    Box::from(format!("{}", msg.to_lowercase()))
}

pub fn get_error(msg: String) -> TaskExecuteError {
    TaskExecuteError::new(&format!("{}", msg))
}

pub fn get_map_item(item: String) -> Result<String, Box<dyn std::error::Error>> {
    let hm = MAP_LOOKUP.lock()?;
    let deploy_res = hm.as_ref().unwrap().get(&item);
    match deploy_res {
        Some(value) => Ok(value.to_owned()),
        None => Err(get_box_error(format!("item {} not set", item).to_owned())),
    }
}

pub fn get_opts(db: String) -> Result<Tree, Box<dyn std::error::Error>> {
    let db_path = get_map_item("db_path".to_owned())?;
    let tree = TreeBuilder::new()
        .with_path(format!("{}/{}.kv", db_path, db).into())
        .with_max_memtable_size(100 * 1024 * 1024)
        .with_block_size(4096)
        .with_level_count(1);
    log::debug!("db {}/{}.kv", db_path, db);
    Ok(tree.build()?)
}

pub async fn get_queue_items(
    db: String,
) -> Result<HashMap<Bytes, FormData>, Box<dyn std::error::Error>> {
    let tree = get_opts(db)?;
    let mut txn = tree.begin()?;
    txn.set_durability(Durability::Immediate);
    let start_date: DateTime<Utc> = format!("2025-01-01T00:01:00Z").parse()?;
    let current_date = Utc::now().naive_utc();
    let updated_date = current_date + Duration::days(1);
    let start = start_date.format("%Y%m%d%H%M%S").to_string();
    let end = updated_date.format("%Y%m%d%H%M%S").to_string();
    log::debug!("[process_queue] {} {}", start.clone(), end.clone());
    let mut hm: HashMap<Bytes, FormData> = HashMap::new();
    let results = txn.range(start.clone(), end.clone(), Some(24))?;
    for (_i, k) in results.enumerate().into_iter() {
        match k {
            Ok(val) => {
                let data_res = val.1;
                match data_res {
                    Some(data) => {
                        let fd: FormData = serde_json::from_slice(&data.to_vec())?;
                        // work with 1 item at a time
                        if hm.len() == 0 {
                            let key = val.0;
                            let b_key = Bytes::from(key.to_owned().to_vec());
                            hm.insert(b_key, fd.clone());
                        }
                    }
                    None => {
                        log::warn!("[process_queue] no value in database");
                    }
                }
            }
            Err(e) => {
                return Err(get_box_error(e.to_string()));
            }
        }
    }
    txn.commit().await?;
    tree.close().await?;
    Ok(hm.clone())
}
