use clap::{Parser, Subcommand};
use serde_derive::{Deserialize, Serialize};

/// rust-secure-exec-service cli struct
#[derive(Parser)]
#[command(name = "secure-exec-service")]
#[command(author = "Luigi Mario Zuccarelli <luzuccar@redhat.com>")]
#[command(version = "0.1.0")]
#[command(about = "A simple secure (HTTPS) service that execute commands asynchrously", long_about = None)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    /// set the loglevel
    #[arg(
        value_enum,
        short,
        long,
        value_name = "loglevel",
        default_value = "info",
        help = "Set the log level [possible values: info, debug, trace]"
    )]
    pub loglevel: Option<String>,

    /// set the mode (client or server)
    #[arg(
        short,
        long,
        value_name = "mode",
        help = "Set the mode [possible values: controller, worker] (required)"
    )]
    pub mode: Option<String>,

    /// server ip address (only for worker)
    #[arg(
        short,
        long,
        value_name = "server-ip",
        help = "The server ip address for the worker to connect to (default 127.0.0.1)"
    )]
    pub server_ip: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// RemoteExecute
    RemoteExecute {
        #[arg(
            short,
            long,
            value_name = "node",
            help = "Deploy to a specific node (hostname of server) or all servers"
        )]
        node: String,
    },
    /// RemoteUpload
    RemoteUpload {
        #[arg(
            short,
            long,
            value_name = "node",
            help = "Deploy to a specific node (hostname of server) or all servers"
        )]
        node: String,
        #[arg(
            short,
            long,
            value_name = "file",
            help = "File to upload to remote server"
        )]
        file: String,
    },
}

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
    #[serde(rename = "prompt")]
    pub contents: String,

    #[serde(rename = "file")]
    pub file_name: String,

    #[serde(rename = "title")]
    pub title: String,

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
