use crate::{
    error::handler::TaskExecuteError,
    handlers::{
        common::get_error,
        execute::{Execute, ExecuteInterface, TaskExecute},
        payload::{Payload, PayloadInterface},
        queue::{Queue, QueueInterface},
    },
    service::validate::validate_jwt,
};
use custom_logger as log;
use http::{HeaderValue, Method, Request, Response, StatusCode};
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct TaskExecuteResponse {
    message: String,
    status: String,
}

fn get_response(status: String, message: String) -> String {
    let response = TaskExecuteResponse { status, message };
    // if we can't serialize this, were in trouble
    serde_json::to_string(&response).unwrap()
}

fn parse_payload(data: &[u8]) -> Result<TaskExecute, TaskExecuteError> {
    log::debug!(
        "[parse_payload] raw data {}",
        String::from_utf8(data.to_vec()).unwrap()
    );
    let te: TaskExecute = serde_json::from_slice(&data).map_err(|e| get_error(e.to_string()))?;
    log::debug!("[parse_payload]  struct {:?}", te);
    Ok(te)
}

#[allow(dead_code)]
fn check_header(hv: Option<&HeaderValue>) -> Result<String, TaskExecuteError> {
    match hv {
        Some(value) => {
            let agent = value.to_str().map_err(|e| get_error(e.to_string()))?;
            Ok(agent.to_string())
        }
        None => {
            return Err(get_error("header for 'agent' not set".to_string()));
        }
    }
}

fn aggregate(data: &[u8]) -> Result<String, TaskExecuteError> {
    let te = parse_payload(data)?;
    validate_jwt(te.spec.token.clone())?;
    let exec_res = Execute::process_task(te)?;
    Ok(exec_res)
}

pub async fn task_service(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let mut response = Response::new(Full::default());
    let req_uri = req.uri().to_string();
    match req.method() {
        // process command service route.
        &Method::POST => {
            let data = req.into_body().collect().await?.to_bytes();
            if req_uri.contains("execute-agent") {
                let result = aggregate(data.as_ref());
                match result {
                    Ok(payload) => {
                        *response.status_mut() = StatusCode::OK;
                        *response.body_mut() = Full::from(payload);
                    }
                    Err(e) => {
                        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                        *response.body_mut() =
                            Full::from(get_response("error".to_owned(), format!("{}", e)));
                    }
                }
            }
            if req_uri.contains("process-queue") {
                log::info!("processing for queue");
                let exec_res = Queue::process_queue().await;
                match exec_res {
                    Ok(_) => {
                        *response.status_mut() = StatusCode::OK;
                        *response.body_mut() = Full::from("db_queue processed ok");
                    }
                    Err(e) => {
                        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                        *response.body_mut() =
                            Full::from(get_response("error".to_owned(), format!("{}", e)));
                    }
                }
            }
            if req_uri.contains("process-payload") {
                let payload_res = Payload::process_payload(&data).await;
                match payload_res {
                    Ok(payload) => {
                        *response.status_mut() = StatusCode::OK;
                        *response.body_mut() = Full::from(payload);
                    }
                    Err(e) => {
                        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                        *response.body_mut() =
                            Full::from(get_response("error".to_owned(), format!("{}", e)));
                    }
                }
            }
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };
    Ok(response)
}
