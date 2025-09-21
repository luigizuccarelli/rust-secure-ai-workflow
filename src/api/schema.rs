use clap::{Parser, Subcommand};

/// rust-secure-exec-service cli struct
#[derive(Parser)]
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
