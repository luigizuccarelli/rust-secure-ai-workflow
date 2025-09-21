use crate::error::handler::TaskExecuteError;
use crate::handlers::common::get_error;
use crate::handlers::common::get_map_item;
use custom_logger as log;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::sign::Verifier;
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufRead;
use std::io::BufReader;
use std::process::{Command, Stdio};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskExecute {
    #[serde(rename = "apiVersion")]
    pub api_version: String,

    #[serde(rename = "kind")]
    pub kind: String,

    #[serde(rename = "spec")]
    pub spec: Spec,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Spec {
    /*
    #[serde(rename = "prompt")]
    pub contents: String,

    #[serde(rename = "file")]
    pub file_name: String,

    #[serde(rename = "title")]
    pub title: String,

    #[serde(rename = "category")]
    pub category: String,
    */
    #[serde(rename = "key")]
    pub key: String,

    #[serde(rename = "token")]
    pub token: String,

    #[serde(rename = "nodes")]
    pub nodes: Vec<NodeExecute>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeExecute {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "agent")]
    pub agent: String,

    #[serde(rename = "args")]
    pub args: Option<Vec<String>>,

    #[serde(rename = "user")]
    pub user: String,

    #[serde(rename = "consoleLog")]
    pub console_log: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct APIResponse {
    #[serde(rename = "status")]
    pub status: String,

    #[serde(rename = "node")]
    pub node: String,

    #[serde(rename = "service")]
    pub service: String,

    #[serde(rename = "text")]
    pub text: String,
}

#[derive(Debug)]
enum ExitStatus {
    OK,
    WARNING,
    ERROR,
}

pub trait ExecuteInterface {
    fn process_task(task_execute: TaskExecute) -> Result<String, TaskExecuteError>;
}

pub struct Execute {}

impl ExecuteInterface for Execute {
    fn process_task(task_execute: TaskExecute) -> Result<String, TaskExecuteError> {
        // the node should always be localhost
        let node = task_execute.spec.nodes[0].to_owned();
        let agent_lookup =
            get_map_item(node.agent.clone()).map_err(|e| get_error(e.to_string()))?;
        let config_file = format!("{}-config.json", agent_lookup.clone());
        let mut exit_status: Option<ExitStatus> = None;
        let mut command = Command::new(agent_lookup.clone());
        let certs_dir =
            get_map_item("certs_dir".to_string()).map_err(|e| get_error(e.to_string()))?;
        verify_artifact(agent_lookup, certs_dir)?;
        // append the key (to do a lookup from database0
        command.arg("--config");
        command.arg(config_file);
        command.arg("--key");
        command.arg(task_execute.spec.key.clone());
        log::debug!("agent command to execute {:?}", command);
        let cmd_res = command.stdout(Stdio::piped()).spawn();
        match cmd_res {
            Ok(res) => {
                let mut out = res.stdout.unwrap();
                let mut reader = BufReader::new(&mut out);
                // we use println and not custom_logger to preserve original output
                println!("");
                loop {
                    let mut line = String::new();
                    let num_bytes = reader.read_line(&mut line).unwrap();
                    match line.clone() {
                        x if x.contains("exit => 0") => {
                            if exit_status.is_none() {
                                exit_status = Some(ExitStatus::OK);
                            }
                        }
                        x if x.contains("exit => 1") => {
                            if exit_status.is_none() {
                                exit_status = Some(ExitStatus::WARNING);
                            }
                        }
                        _ => {
                            // dont set this in the loop
                            // it will always set to ERROR
                        }
                    }
                    if num_bytes == 0 {
                        println!("=> end of stream\n");
                        break;
                    }
                    print!("{}", line);
                }
                if exit_status.is_none() {
                    exit_status = Some(ExitStatus::ERROR);
                }
                match exit_status.unwrap() {
                    ExitStatus::OK => {
                        log::info!("[process_task] agent {} executed successfully", node.agent);
                    }
                    ExitStatus::WARNING => {
                        let err = TaskExecuteError::new(&format!(
                            "[process_task] agent {} executed with warning",
                            node.agent
                        ));
                        log::warn!("{}", err.to_string());
                        return Err(err);
                    }
                    ExitStatus::ERROR => {
                        let err =
                            TaskExecuteError::new(&format!("command failed : {} ", node.agent));
                        log::error!("[process_task] {}", err.to_string().to_lowercase());
                        return Err(err);
                    }
                }
            }
            Err(err) => {
                let task_err = TaskExecuteError::new(&format!(
                    "command failed : {}",
                    err.to_string().to_lowercase()
                ));
                log::error!("[process_task] {}", err.to_string().to_lowercase());
                return Err(task_err);
            }
        }
        let json_res = serde_json::to_string(&task_execute);
        match json_res {
            Ok(json) => Ok(json),
            Err(e) => Err(get_error(e.to_string())),
        }
    }
}

fn verify_artifact(name: String, certs_dir: String) -> Result<(), TaskExecuteError> {
    // Verify the data
    let mut tar_buf = vec![];
    let mut tgz_file =
        File::open(name.clone()).map_err(|e| TaskExecuteError::new(&format!("{}", e)))?;
    tgz_file
        .read_to_end(&mut tar_buf)
        .map_err(|e| TaskExecuteError::new(&format!("{}", e)))?;
    log::debug!("opened tgz file {:?}", tgz_file);
    let mut f_prv = File::open(&format!("{}/public.pem", certs_dir))
        .map_err(|e| TaskExecuteError::new(&format!("[verify_artifact] public.pem {}", e)))?;
    log::debug!("opened public key file {:?}", f_prv);
    let mut buf = vec![];
    f_prv.read_to_end(&mut buf).map_err(|e| {
        TaskExecuteError::new(&format!("[verify_artifact] reading to buffer {}", e))
    })?;
    let public_key = PKey::public_key_from_pem(&buf)
        .map_err(|e| TaskExecuteError::new(&format!("[verify_artifact] public kley {}", e)))?;
    let mut sig = File::open(format!("{}-signature", name)).map_err(|_| {
        TaskExecuteError::new(&format!(
            "[verify_artifact] could not verify {} has it been signed ?",
            name
        ))
    })?;
    log::debug!("signature file {:?}", sig);
    let mut signature_buf = vec![];
    sig.read_to_end(&mut signature_buf)
        .map_err(|e| TaskExecuteError::new(&format!("[verify_artifact] signature buffer {}", e)))?;
    let mut verifier = Verifier::new(MessageDigest::sha256(), &public_key)
        .map_err(|e| TaskExecuteError::new(&format!("[verify_artifact] verifier {}", e)))?;
    verifier
        .update(&tar_buf)
        .map_err(|e| TaskExecuteError::new(&format!("[verify_artifact] verifier update {}", e)))?;
    let _res = verifier
        .verify(&signature_buf)
        .map_err(|e| TaskExecuteError::new(&format!("[verify_artifact] verifier verify {}", e)))?;
    log::debug!("artifact {} verified", name);
    return Ok(());
}
