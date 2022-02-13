use clap::Parser;
use tokio::{
    runtime::{Builder, Runtime},
    sync::oneshot,
};

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
    /// Specify run mode: 's' is for static file serve, otherwise is Blog backend
    #[clap(short, long)]
    mode: Option<String>,

    /// Specify listening address, default value is '127.0.0.1'
    #[clap(short, long)]
    address: Option<String>,

    /// Specify listening port, default value is '9270'
    #[clap(short, long)]
    port: Option<String>,
}

fn main() -> result::Result<()> {
    let args = Args::parse();

    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .thread_name("Songday-blog-service")
        .thread_stack_size(1024 * 1024)
        .build()?;

    let (tx, rx) = oneshot::channel::<()>();

    runtime.spawn(async {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                println!("Shutting down web server...");
                match tx.send(()) {
                    Ok(()) => {},
                    Err(_) => println!("the receiver dropped"),
                }
            },
            Err(e) => {
                eprintln!("{}", e);
            },
        }
    });

    let run_mode = if args.mode.is_none() {
        let mut line = String::with_capacity(16);
        println!("指定运行模式, 直接回车是博客后台，按 s 是静态文件服务");
        println!("Specify run mode, default (Press 'Enter' directly) is Blog backend, `s` is static file serve");
        let _b1 = std::io::stdin().read_line(&mut line).unwrap();
        // println!("Hello , {}", line);
        // println!("no of bytes read , {}", b1);
        String::from(line.trim())
    } else {
        args.mode.unwrap()
    };

    // let mut address = if args.port.is_none() {
    //     String::from("127.0.0.1")
    // } else {
    //     args.address.unwrap()
    // };
    let mut address = args.address.unwrap_or(String::from("127.0.0.1"));

    // let port = if args.port.is_none() {
    //     String::from("9270")
    // } else {
    //     args.port.unwrap()
    // };
    let port = args.port.as_ref().unwrap_or("9270");

    address.push_str(":");
    address.push_str(port);

    if run_mode.eq("s") {
        println!("Creating server instance...");
        let server = runtime.block_on(service::server::create_static_file_server(&address, rx))?;

        println!("Starting static file server...");
        runtime.block_on(server);
    } else {
        println!("Initializing database connection...");
        runtime.block_on(db::init_datasource());

        println!("Creating server instance...");
        let server = runtime.block_on(service::server::create_blog_server(&address, rx))?;

        runtime.spawn(service::status::scanner());

        println!("Starting blog backend server...");
        runtime.block_on(server);

        // println!("Starting web server...");
        // let server = runtime.block_on(async { server::create_server("127.0.0.1:9270", rx).await.unwrap() });
        // runtime.block_on(server);

        println!("Closing database connections...");
        runtime.block_on(db::shutdown());
    }

    println!("Bye...");

    Ok(())
    /*
    tokio::spawn(async move {
        let r = tokio::signal::ctrl_c().await;
        println!("ctrl-c received!");
    });
    println!("Starting web server...");
    server::start("127.0.0.1:9270").await
    */
}
