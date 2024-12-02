use std::time::Duration;

#[tokio::main]
async fn main() {
    let task = tokio::spawn(spawn_task());
    task.await.unwrap();
}

async fn spawn_task() {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    interval.tick().await;
    loop {
        interval.tick().await;
        tokio::spawn(do_the_thing());
    }
}

async fn do_the_thing() {
    // be busy for a while
    std::thread::sleep(Duration::from_secs(1));
    // yield
    // tokio::time::sleep(Duration::from_secs(1)).await;

    println!("Doing the thing");
}
