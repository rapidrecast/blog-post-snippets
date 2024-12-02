use tower::ServiceBuilder;
use crate::service::MyTowerService;

mod service;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let listener = tokio::net::TcpListener::from_std(listener).unwrap();
    let bind_addr = listener.local_addr().unwrap();
    println!("Listening on http://{}", bind_addr);
    loop {
        let (tcp_stream, addr) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            println!("Received connection from {addr:?}, spawning");
            let tcp_stream = hyper_util::rt::TokioIo::new(tcp_stream);
            let http1_server = hyper::server::conn::http1::Builder::new()
                .serve_connection(tcp_stream, MyTowerService{});
            let result = http1_server.await;
            if let Err(e) = result {
                eprintln!("{:?}", e);
            }
            println!("Finished serving connection for {addr:?}");
        });
    }
}

