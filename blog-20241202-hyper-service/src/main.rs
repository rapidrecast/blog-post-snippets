use crate::bad_service::BadTowerService;
use crate::good_service::GoodTowerService;
use tokio::net::TcpListener;

mod bad_service;
mod error;
mod good_service;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let listener = TcpListener::from_std(listener).unwrap();
    let bind_addr = listener.local_addr().unwrap();
    println!("Listening on http://{}", bind_addr);
    #[cfg(feature = "bad-impl")]
    bad_solution(listener).await;
    #[cfg(not(feature = "bad-impl"))]
    good_solution(listener).await;
}

async fn bad_solution(listener: TcpListener) {
    loop {
        let (tcp_stream, addr) = listener.accept().await.unwrap();
        println!("Received connection from {addr:?}, spawning");
        tokio::spawn(async move {
            let tcp_stream = hyper_util::rt::TokioIo::new(tcp_stream);
            let http1_server = hyper::server::conn::http1::Builder::new()
                .keep_alive(false)
                .serve_connection(tcp_stream, BadTowerService {});
            let result = http1_server.await;
            if let Err(e) = result {
                eprintln!("Error: {:?}", e);
            }
            println!("Finished serving connection for {addr:?}");
        });
    }
}

async fn good_solution(listener: TcpListener) {
    loop {
        let (tcp_stream, addr) = listener.accept().await.unwrap();
        println!("Received connection from {addr:?}, spawning");
        tokio::spawn(async move {
            let tcp_stream = hyper_util::rt::TokioIo::new(tcp_stream);
            let result = hyper::server::conn::http1::Builder::new()
                .keep_alive(false)
                .serve_connection(tcp_stream, GoodTowerService {}).await;
            if let Err(e) = result {
                eprintln!("Error: {:?}", e);
            }
            println!("Finished serving connection for {addr:?}");
        });
        // tokio::task::yield_now().await;
    }
}