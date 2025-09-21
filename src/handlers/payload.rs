use crate::handlers::common::{get_box_error, get_map_item, get_opts, get_queue_items};
use crate::handlers::execute::{NodeExecute, Spec, TaskExecute};
use custom_logger as log;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Method, Request, Uri};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use serde_derive::{Deserialize, Serialize};
use surrealkv::Durability;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserData {
    #[serde(rename = "user")]
    pub user: String,

    #[serde(rename = "session_id")]
    pub session_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Token {
    #[serde(rename = "access_token")]
    pub access_token: String,

    #[serde(rename = "token_type")]
    pub token_type: String,
}

pub trait PayloadInterface {
    async fn process_payload(data: &[u8]) -> Result<String, Box<dyn std::error::Error>>;
}

pub struct Payload {}

impl PayloadInterface for Payload {
    async fn process_payload(data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
        let s_data = str::from_utf8(data)?;
        let mut payload = String::new();
        let tree = get_opts("queue".to_string())?;
        let mut txn = tree.begin()?;
        txn.set_durability(Durability::Immediate);
        let hm_queue = get_queue_items("queue".to_string()).await?;
        let token_url = get_map_item("token_url".to_string())?;
        log::debug!("[process_payload] token_url {}", token_url);
        for (k, v) in hm_queue {
            log::debug!("[process_payload] body data {}", s_data);
            if v.category.contains(s_data) {
                let mut te = get_taskexecute();
                let token = process_token(v.credentials.clone(), token_url.clone()).await?;
                te.spec.token = token;
                te.spec.key = String::from_utf8(k.to_vec())?;
                payload = serde_json::to_string(&te)?;
            } else {
                return Err(get_box_error(format!(
                    "this is a {} category, please ensure the source trigger matches '{}'",
                    v.category, v.category
                )));
            }
        }
        log::debug!("[process_payload] payload {}", payload);
        Ok(payload)
    }
}

fn get_taskexecute() -> TaskExecute {
    let nodes = NodeExecute {
        name: "localhost".to_string(),
        agent: "none".to_string(),
        user: "none".to_string(),
        args: None,
        console_log: true,
    };
    let vec_nodes = vec![nodes];
    let spec = Spec {
        key: "none".to_string(),
        token: "none".to_string(),
        nodes: vec_nodes.clone(),
    };
    let te = TaskExecute {
        api_version: "api.taskexecute.io/v1".to_string(),
        kind: "TaskExecute".to_string(),
        spec,
    };
    te
}

async fn process_token(
    user: String,
    base_url: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let https = HttpsConnector::new();
    let client: Client<_, Full<Bytes>> = Client::builder(TokioExecutor::new()).build(https);
    if base_url == "" {
        return Err(Box::from(format!("{}:token url not set", 422)));
    } else {
        let uri: Uri = format!("{}", base_url).parse()?;
        log::debug!("[process_token] url {}", uri);
        let ud = UserData {
            user,
            session_id: "123456".to_string(),
        };
        let payload = serde_json::to_string(&ud)?;
        log::debug!("[process_token] payload {}", payload);

        let req: Request<Full<Bytes>> = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .body(Full::from(payload))?;

        let future = client.request(req).await?;
        let response = future.into_body().collect().await?.to_bytes();
        let token: Token = serde_json::from_slice(&response)?;
        return Ok(token.access_token);
    }
}

#[allow(dead_code)]
async fn call_agent(
    payload: String,
    base_url: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let https = HttpsConnector::new();
    let client: Client<_, Full<Bytes>> = Client::builder(TokioExecutor::new()).build(https);
    if base_url == "" {
        return Err(Box::from(format!("{}:token url not set", 422)));
    } else {
        let uri: Uri = format!("{}", base_url).parse()?;
        log::debug!("[call_agent] url {}", uri);
        let req: Request<Full<Bytes>> = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .body(Full::from(payload))?;

        let future = client.request(req).await?;
        let response = future.into_body().collect().await?.to_bytes();
        return Ok(String::from_utf8(response.to_vec()).unwrap());
    }
}
