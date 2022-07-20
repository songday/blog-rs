use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::*;
use std::fs;
pub fn load_config(args: &mut Args) -> Result<()> {
    let data = fs::read_to_string(args.config.as_ref().unwrap()).unwrap();
    let v: Args = serde_json::from_str(data.as_str())?;
    *args = v;
    // let b = Box::leak(Box::new(v));
    // args = b;
    Ok(())
}
/// Simple blog backend
#[derive(Parser, Serialize, Deserialize)]
#[clap(name = "Songday blog backend", author, version, about, long_about = None)]
pub struct Args {
    #[clap(long, value_parser)]
    /// Specify config path, e.g.: ./config.json
    pub config: Option<String>,
    /// Specify run mode: 'static' is for static file serve, 'blog' is blog warp server mode
    #[clap(long, value_parser)]
    pub mode: Option<String>,

    /// HTTP Server Settings
    /// Specify http listening address, e.g.: 0.0.0.0 or [::] or 127.0.0.1 or other particular ip, default is '127.0.0.1'
    #[clap(long, default_value = "127.0.0.1", value_parser)]
    pub ip: String,

    /// Specify listening port, default value is '80'
    #[clap(long, default_value_t = 80, value_parser)]
    pub port: u16,

    /// Enable HTTPS Server
    #[clap(long, value_parser)]
    pub https_enabled: bool,

    /// Cert file path, needed by https
    #[clap(long, value_parser)]
    pub cert_path: Option<String>,

    /// Key file path, needed by https
    #[clap(long, value_parser)]
    pub key_path: Option<String>,

    /// Specify HTTPS listening port, default value is '443'
    #[clap(long, value_parser, default_value_t = 443)]
    pub https_port: u16,

    /// Enable HSTS Redirect Server
    #[clap(long, value_parser)]
    pub hsts_enabled: bool,

    /// Hostname for CORS
    #[clap(long, value_parser)]
    pub cors_host: Option<String>,
}
