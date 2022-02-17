use clap::Parser;
use tokio::{
    runtime::{Builder, Runtime},
    sync::oneshot,
};
use futures::{future::{BoxFuture, join_all}, Future};
use std::{net::SocketAddr};
use blog_backend::{db, service, util::result};

#[derive(Debug, PartialEq)]
enum RunMode {
    Blog,
    StaticBackend,
}

/// Simple blog backend
#[derive(Parser, Debug)]
#[clap(name = "Songday blog backend", author, version, about, long_about = None)]
struct Args {
    /// Specify run mode: 'static' is for static file serve, 'blog' is blog warp server mode
    #[clap( long , default_value ="blog")]
    mode: String,

   /// Enable HTTP Server
    #[clap(long)]
    http_enable:bool,
    /// Specify http listening address, default value is '127.0.0.1'
    #[clap(long , default_value="127.0.0.1")]
    http_address: String,

    /// Specify listening port, default value is '80'
    #[clap( long, default_value_t=80)]
    http_port: u16,
    /// Enable HTTPS Server
    #[clap(long)]
    https_enable:bool,
    /// Specify https listening address, default value is '127.0.0.1'
    #[clap( long , default_value="127.0.0.1")]
    https_address: String,

    /// Specify listening port, default value is '443'
    #[clap( long, default_value_t=443)]
    https_port: u16,
    /// Enable HSTS Redirect Server
    #[clap(long)]
    hsts_enable:bool,
}
pub struct HttpConfig{
    pub enabled:bool,
}
pub struct HttpsConfig{
    pub enabled:bool,
    pub hsts:bool,
}
fn main() -> result::Result<()> {
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
            },
            Err(e) => {
                eprintln!("{}", e);
            },
        }
    });
    runtime.spawn(async {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                println!("Shutting down web server...");
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



    let run_mode = args.mode;
    if run_mode.eq("static") {
        println!("Creating server instance...");
        let http_address=args.http_address.parse::<SocketAddr>()?;
        let server = runtime.block_on(service::server::create_static_file_server(http_address, rx1))?;

        println!("Starting static file server...");
        runtime.block_on(server);
    } else if run_mode.eq("blog"){
        let mut https_address = args.https_address;
        https_address.push_str(":");
        https_address.push_str(&args.https_port.to_string());
        let https_address=https_address.parse::<SocketAddr>()?;
        let mut http_address = args.http_address;
        http_address.push_str(":");
        http_address.push_str(&args.http_port.to_string());
        let http_address=http_address.parse::<SocketAddr>()?;
        let https_config=HttpsConfig{
            enabled:args.https_enable,
            hsts:args.hsts_enable,
           };
        let http_config=HttpConfig{
            enabled:args.http_enable,
           };
        println!("Initializing database connection...");
        runtime.block_on(db::init_datasource());
        
        println!("Creating server instance...");
        let mut vec:Vec<BoxFuture<()>> = Vec::new();
        if http_config.enabled{
            if https_config.hsts{
                let server=runtime.block_on( service::server::create_blog_server_hsts(http_address, rx1));
                vec.push(Box::pin(server.unwrap()));              
                println!("Creating HSTS Redirect server instance...");
            }else{
                let server= runtime.block_on(service::server::create_blog_server(http_address, rx1));
                println!("Starting http blog backend server...");
                vec.push(Box::pin(server.unwrap()));
            }
        }
        if https_config.enabled{
            let server=runtime.block_on(service::server::create_tls_blog_server(https_address, rx2));
            println!("Starting https blog backend server...");
            vec.push(Box::pin(server.unwrap()));
        }
        let server=join_all(vec);

        runtime.block_on(server);
        println!("Closing database connections...");
        runtime.block_on(db::shutdown());
    }

    println!("Bye...");

    Ok(())
}
