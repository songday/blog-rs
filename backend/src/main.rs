#![recursion_limit = "256"]

use std::net::SocketAddr;

use blog_backend::{db, service, util::result,config::{config_loader, self}};
use clap::Parser;
use futures::future::{join_all, BoxFuture};
use tokio::{
    runtime::{Builder, Runtime},
    sync::broadcast,
};



fn main() -> result::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "access-log=info");
    }
    pretty_env_logger::init();
    
    let mut args = crate::config_loader::Args::parse();
    if args.config.is_some(){
        let config_result = config_loader::load_config(&mut args);
        match config_result{
            Ok(_)=>{
                println!("Config Loaded")
            },
            Err(_)=>{
                panic!("Config Invalid!");
            },
            _=>()
        }
    }
    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .thread_name("Songday-blog-service")
        .thread_stack_size(1024 * 1024)
        .build()?;

    let (tx, rx1) = broadcast::channel(2);
    let rx2 = tx.subscribe();
    runtime.spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                println!("Shutting down web server...");
                match tx.send(()) {
                    Ok(_) => {},
                    Err(_) => println!("the receiver dropped"),
                }
            },
            Err(e) => {
                eprintln!("{}", e);
            },
        }
    });

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
