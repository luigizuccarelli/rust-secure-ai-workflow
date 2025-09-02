use crate::api::schema::{NodeExecute, Spec, TaskExecute};
use crate::command::process::execute;
use crate::error::handler::TaskExecuteError;
use custom_logger as log;
use http::{Method, Request, Response, StatusCode};
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use serde::{Deserialize, Serialize};
use std::str;
use std::{env, fs};

#[derive(Deserialize, Serialize, Debug)]
struct Claims {
    user: String,
    iss: String,
    sub: String,
    aud: String,
    exp: u64,
}

#[derive(Debug, Serialize)]
struct TaskExecuteResponse {
    message: String,
    status: String,
}

pub fn get_error(msg: String) -> TaskExecuteError {
    TaskExecuteError::new(&format!("{}", msg.to_lowercase()))
}

pub fn get_response(status: String, message: String) -> String {
    let response = TaskExecuteResponse { status, message };
    // if we can't serialize this, were in trouble
    serde_json::to_string(&response).unwrap()
}

fn validate_jwt(token: String) -> Result<(), TaskExecuteError> {
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.set_audience(&vec!["samcopai"]);
    validation.set_issuer(&vec!["https://samcopai.com"]);
    validation.set_required_spec_claims(&["iss", "aud", "exp", "sub", "user"]);
    let jwt_secret = match env::var("JWT_SECRET") {
        Ok(var) => var,
        Err(_) => "secret".to_string(),
    };
    let secret = jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes());
    // decode token
    let jwt = jsonwebtoken::decode::<Claims>(&token, &secret, &validation)
        .map_err(|e| get_error(e.to_string()))?;
    log::trace!("jwt details {:?}", jwt);
    Ok(())
}

fn get_payload() -> Result<TaskExecute, TaskExecuteError> {
    let payload =
        fs::read_to_string("./staging/payload.json").map_err(|e| get_error(e.to_string()))?;
    let te: TaskExecute = serde_json::from_str(&payload).map_err(|e| get_error(e.to_string()))?;
    Ok(te)
}

fn execute_task(agent: String, te: Option<TaskExecute>) -> Result<String, TaskExecuteError> {
    let mut tasks: TaskExecute;
    if te.is_none() {
        let ne = NodeExecute {
            name: "".to_string(),
            agent,
            args: None,
            user: "".to_string(),
            console_log: true,
        };
        let vec_nodes = vec![ne.clone()];
        let spec = Spec {
            contents: "".to_string(),
            title: "".to_string(),
            file_name: "".to_string(),
            token: "".to_string(),
            nodes: vec_nodes,
        };
        tasks = TaskExecute {
            api_version: "api.taskexecute.io/v1".to_string(),
            kind: "TaskExecute".to_string(),
            spec,
        }
    } else {
        // only one node for our execution
        // so the index will always be 0
        tasks = te.unwrap();
        tasks.spec.nodes[0].agent = agent.to_owned();
    }
    log::debug!("task execute details {:?}", tasks);
    // no async - just wait for response
    let result = execute(tasks)?;
    Ok(result)
}

pub async fn process(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let mut response = Response::new(Full::default());
    match (req.method(), req.uri().path()) {
        // process command service route.
        (&Method::POST, "/execute-agent") => {
            // check jwt
            let agent = req.headers().get("agent");
            // get agent header
            match agent {
                Some(val) => {
                    // get payload
                    log::debug!("agent ok");
                    let payload_res = get_payload();
                    match payload_res {
                        Ok(payload) => {
                            log::debug!("payload ok");
                            let validate = validate_jwt(payload.spec.token.clone());
                            match validate {
                                Ok(_) => {
                                    // execute task
                                    let exec_res = execute_task(
                                        val.to_str().unwrap().to_owned(),
                                        Some(payload),
                                    );
                                    match exec_res {
                                        Ok(exec) => {
                                            *response.status_mut() = StatusCode::OK;
                                            if exec == "ok" {
                                                *response.body_mut() = Full::from(get_response(
                                                    "ok".to_owned(),
                                                    format!(
                                                        "task {} executed successfully",
                                                        "agent"
                                                    ),
                                                ));
                                            } else {
                                                // could still be executing
                                                *response.body_mut() = Full::from(get_response(
                                                    "ok".to_owned(),
                                                    format!("{}", exec),
                                                ));
                                            }
                                        }
                                        Err(e) => {
                                            *response.status_mut() =
                                                StatusCode::INTERNAL_SERVER_ERROR;
                                            *response.body_mut() = Full::from(get_response(
                                                "interanl server error".to_owned(),
                                                format!("error : {}", e),
                                            ));
                                        }
                                    }
                                }
                                Err(e) => {
                                    *response.status_mut() = StatusCode::FORBIDDEN;
                                    *response.body_mut() = Full::from(get_response(
                                        "forbidden".to_owned(),
                                        format!("error : {}", e),
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Full::from(get_response(
                                "internal server error".to_owned(),
                                format!("error : {}", e),
                            ));
                        }
                    }
                }
                None => {
                    *response.status_mut() = StatusCode::BAD_REQUEST;
                    *response.body_mut() = Full::from(get_response(
                        "bad request".to_owned(),
                        "error : agent header not set".to_owned(),
                    ));
                }
            }
        }
        (&Method::POST, "/process-payload") => {
            let agent = req.headers().get("agent");
            // get agent header
            match agent {
                Some(val) => {
                    let exec_res = execute_task(val.to_str().unwrap().to_owned(), None);
                    match exec_res {
                        Ok(exec) => {
                            *response.status_mut() = StatusCode::OK;
                            *response.body_mut() =
                                Full::from(get_response("ok".to_owned(), format!("{}", exec)));
                        }
                        Err(e) => {
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Full::from(get_response(
                                "internal server error".to_owned(),
                                format!("error : {}", e),
                            ));
                        }
                    }
                }
                None => {}
            }
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };
    Ok(response)
}
