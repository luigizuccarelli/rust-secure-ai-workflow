use crate::api::schema::TaskExecute;
use crate::error::handler::TaskExecuteError;
use crate::MAP_LOOKUP;
use custom_logger as log;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::sign::Verifier;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug)]
enum ExitStatus {
    OK,
    WARNING,
    ERROR,
}

pub fn execute(task_execute: TaskExecute) -> Result<String, TaskExecuteError> {
    let response = "task process completed successfully".to_string();

    if Path::new("semaphore.pid").exists() {
        return Ok("a process is still executing, please try again later".to_string());
    }

    let _ = fs::write("semaphore.pid", "true");
    // the node should always be localhost
    let node = task_execute.spec.nodes[0].to_owned();
    let mut exit_status: Option<ExitStatus> = None;
    // lookup the agent script
    let hm = MAP_LOOKUP.lock().unwrap();
    let res = hm.as_ref().unwrap().get(&task_execute.spec.nodes[0].agent);
    let script;
    match res {
        Some(value) => {
            if value == "" {
                let _ = fs::remove_file("semaphore.pid");
                let e = TaskExecuteError::new(&format!(
                    "agent {} script has not been defined ",
                    node.agent
                ));
                return Err(e);
            }
            script = value.to_string();
        }
        None => {
            let _ = fs::remove_file("semaphore.pid");
            let e = TaskExecuteError::new(&format!(
                "agent {} script has not been defined ",
                node.agent
            ));
            return Err(e);
        }
    }

    let mut command = Command::new(script.to_string());
    log::debug!("script {}", script.to_string());
    // bypass verification if getting a token
    let verified = verify_artifact(
        script.clone(),
        hm.as_ref().unwrap().get("certs_dir").unwrap().to_string(),
    );
    match verified {
        Ok(_) => {
            // all good just continue
            // artifact was signed
        }
        Err(e) => {
            let _ = fs::remove_file("semaphore.pid");
            log::error!("[local_execute] {}", e.to_string().to_lowercase());
            let e = TaskExecuteError::new(&format!("verifying artifact {} {}", node.agent, e));
            return Err(e);
        }
    }
    command.arg(task_execute.spec.contents.clone());
    command.arg(task_execute.spec.file_name.clone());
    command.arg(task_execute.spec.title.clone());
    let cmd_res = command.stdout(Stdio::piped()).spawn();
    match cmd_res {
        Ok(res) => {
            let mut out = res.stdout.unwrap();
            let mut reader = BufReader::new(&mut out);
            // we use println and not custom_logger to preserve original output
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
                    println!("=> end of stream");
                    break;
                }
                // this preserves colors
                print!("{}", line);
            }
            if exit_status.is_none() {
                exit_status = Some(ExitStatus::ERROR);
            }
            match exit_status.unwrap() {
                ExitStatus::OK => {
                    log::info!("[local_execute] agent {} executed successfully", node.agent);
                }
                ExitStatus::WARNING => {
                    let _ = fs::remove_file("semaphore.pid");
                    let err = TaskExecuteError::new(&format!(
                        "[local_execute] agent {} execute witn no content",
                        node.agent
                    ));
                    log::warn!("[local_execute] no content");
                    return Err(err);
                }
                ExitStatus::ERROR => {
                    let _ = fs::remove_file("semaphore.pid");
                    let err = TaskExecuteError::new(&format!("command failed : {} ", node.agent));
                    log::error!("[local_execute] {}", err.to_string().to_lowercase());
                    return Err(err);
                }
            }
        }
        Err(err) => {
            let _ = fs::remove_file("semaphore.pid");
            let task_err = TaskExecuteError::new(&format!(
                "command failed : {}",
                err.to_string().to_lowercase()
            ));
            log::error!("[local_execute] {}", err.to_string().to_lowercase());
            return Err(task_err);
        }
    }
    let _ = fs::remove_file("semaphore.pid");
    Ok(response)
}

pub fn verify_artifact(name: String, certs_dir: String) -> Result<(), TaskExecuteError> {
    // Verify the data
    let mut tar_buf = vec![];
    let mut tgz_file =
        File::open(name.clone()).map_err(|e| TaskExecuteError::new(&format!("{}", e)))?;
    tgz_file
        .read_to_end(&mut tar_buf)
        .map_err(|e| TaskExecuteError::new(&format!("{}", e)))?;
    log::debug!("opened tgz file {:?}", tgz_file);
    let mut f_prv = File::open(&format!("{}/public.pem", certs_dir))
        .map_err(|e| TaskExecuteError::new(&format!("{}", e)))?;
    log::debug!("opened public key file {:?}", f_prv);
    let mut buf = vec![];
    f_prv
        .read_to_end(&mut buf)
        .map_err(|e| TaskExecuteError::new(&format!("{}", e)))?;
    let public_key =
        PKey::public_key_from_pem(&buf).map_err(|e| TaskExecuteError::new(&format!("{}", e)))?;
    let mut sig = File::open(format!("{}-signature", name)).map_err(|_| {
        TaskExecuteError::new(&format!("could not verify {} has it been signed ?", name))
    })?;
    log::debug!("signature file {:?}", sig);
    let mut signature_buf = vec![];
    sig.read_to_end(&mut signature_buf)
        .map_err(|e| TaskExecuteError::new(&format!("{}", e)))?;
    let mut verifier = Verifier::new(MessageDigest::sha256(), &public_key)
        .map_err(|e| TaskExecuteError::new(&format!("{}", e)))?;
    verifier
        .update(&tar_buf)
        .map_err(|e| TaskExecuteError::new(&format!("{}", e)))?;
    let _res = verifier
        .verify(&signature_buf)
        .map_err(|e| TaskExecuteError::new(&format!("{}", e)))?;
    log::debug!("artifact {} verified", name);
    return Ok(());
}
