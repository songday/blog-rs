#![recursion_limit = "256"]

use blog_backend::{db, service, util::result};
use clap::Parser;
use futures::{
    future::{join_all, BoxFuture},
    Future,
};
use std::net::SocketAddr;
use tokio::{
    runtime::{Builder, Runtime},
    sync::oneshot,
};

/// Simple blog backend
#[derive(Parser)]
#[clap(name = "Songday blog backend", author, version, about, long_about = None)]
struct Args {
    /// Specify run mode: 'static' is for static file serve, 'blog' is blog warp server mode
    #[clap(long, value_parser)]
    mode: Option<String>,

    /// HTTP Server Settings
    /// Specify http listening address, e.g.: 0.0.0.0 or [::] or 127.0.0.1 or other particular ip, default is '127.0.0.1'
    #[clap(long, default_value = "127.0.0.1", value_parser)]
    ip: String,

    /// Specify listening port, default value is '80'
    #[clap(long, default_value_t = 80, value_parser)]
    port: u16,

    /// Enable HTTPS Server
    #[clap(long, value_parser)]
    https_enabled: bool,

    /// Cert file path, needed by https
    #[clap(long, value_parser)]
    cert_path: Option<String>,

    /// Key file path, needed by https
    #[clap(long, value_parser)]
    key_path: Option<String>,

    /// Specify HTTPS listening port, default value is '443'
    #[clap(long, value_parser, default_value_t = 443)]
    https_port: u16,

    /// Enable HSTS Redirect Server
    #[clap(long, value_parser)]
    hsts_enabled: bool,

    /// Hostname for CORS
    #[clap(long, value_parser)]
    cors_host: Option<String>,
}

fn main() -> result::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "access-log=info");
    }
    pretty_env_logger::init();

    let args = Args::parse();

    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .thread_name("Songday-blog-service")
        .thread_stack_size(1024 * 1024)
        .build()?;

    let (tx1, rx1) = oneshot::channel::<()>();
    let (tx2, rx2) = oneshot::channel::<()>();

    runtime.spawn(async {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                println!("Shutting down web server...");
                match tx1.send(()) {
                    Ok(()) => {},
                    Err(_) => println!("the receiver dropped"),
                }
                match tx2.send(()) {
                    Ok(()) => {},
                    Err(_) => println!("the receiver dropped"),
                }
            },
            Err(e) => {
                eprintln!("{}", e);
            },
        }
    });
    // runtime.spawn(async {
    //     match tokio::signal::ctrl_c().await {
    //         Ok(()) => {
    //             println!("Shutting down web server...");
    //             match tx2.send(()) {
    //                 Ok(()) => {},
    //                 Err(_) => println!("the receiver dropped"),
    //             }
    //         },
    //         Err(e) => {
    //             eprintln!("{}", e);
    //         },
    //     }
    // });

    let mut addr = String::from(&args.ip);
    addr.push_str(":");
    addr.push_str(&args.port.to_string());
    let http_address = addr.parse::<SocketAddr>()?;

    if args.mode.is_some() && args.mode.unwrap().eq("static") {
        println!("Creating static file server instance...");
        let server = runtime.block_on(service::server::create_static_file_server(http_address, rx1))?;

        println!("Starting static file server...");
        runtime.block_on(server);
    } else {
        println!("Initializing database connection...");
        runtime.block_on(db::init_datasource());

        println!("Creating server instance...");
        let mut servers: Vec<BoxFuture<()>> = Vec::new();

        let server = runtime.block_on(service::server::create_blog_server(http_address, rx1, &args.cors_host));
        println!("Starting http blog backend server...");
        servers.push(Box::pin(server.unwrap()));

        if args.https_enabled {
            let mut addr = String::from(&args.ip);
            addr.push_str(":");
            addr.push_str(&args.https_port.to_string());
            let https_address = addr.parse::<SocketAddr>()?;

            let cert_path = &args.cert_path.unwrap();
            let key_path = &args.key_path.unwrap();

            if args.hsts_enabled {
                let server = service::server::create_tls_blog_server_with_hsts(
                    https_address,
                    rx2,
                    cert_path,
                    key_path,
                    &args.cors_host,
                );
                let server = runtime.block_on(server);
                println!("Starting https blog backend server...");
                servers.push(Box::pin(server.unwrap()));
            } else {
                let server =
                    service::server::create_tls_blog_server(https_address, rx2, cert_path, key_path, &args.cors_host);
                let server = runtime.block_on(server);
                println!("Starting https blog backend server...");
                servers.push(Box::pin(server.unwrap()));
            }
        }
        let server = join_all(servers);

        runtime.block_on(server);
        println!("Closing database connections...");
        runtime.block_on(db::shutdown());
    }

    println!("Bye...");

    Ok(())
}
