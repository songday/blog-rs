use tokio::{
    runtime::{Builder, Runtime},
    sync::oneshot,
};

use blog_backend::{db, serve::server, service};
use blog_backend::util::result;

fn main() -> result::Result<()> {
    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .thread_name("songday-web-service")
        .thread_stack_size(64 * 1024)
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

    println!("Initializing database connection...");
    runtime.block_on(db::init_datasource());

    println!("Creating server instance...");
    let server = runtime.block_on(server::create_warp_server("127.0.0.1:9270", rx))?;

    runtime.spawn(service::status::scanner());

    println!("Starting web server...");
    runtime.block_on(server);

    // println!("Starting web server...");
    // let server = runtime.block_on(async { server::create_server("127.0.0.1:9270", rx).await.unwrap() });
    // runtime.block_on(server);

    println!("Closing database connections...");
    runtime.block_on(db::shutdown());

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
