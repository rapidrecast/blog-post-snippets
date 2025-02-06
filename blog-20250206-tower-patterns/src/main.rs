use crate::pattern_basic::BasicPatternLayer;
use crate::pattern_io::IoTowerLayer;
use tokio::io::{simplex, AsyncReadExt, AsyncWriteExt};
use tower::{service_fn, Service, ServiceBuilder};

mod pattern_io;
mod pattern_chan;
mod pattern_basic;
mod pattern_handler;
mod error;

#[tokio::main]
async fn main() {
    // Basic pattern of a layer accepting an input and giving an output
    // NOTE: this pattern is applied both to input and output.
    // In reality, it would be more likely to be applied to just one of them.
    // The other patterns would apply to the other side.
    let mut basic_service = ServiceBuilder::new().layer(BasicPatternLayer::<u8, u16, u32>::default()).service(service_fn(basic_service_fn));
    let input: u8 = 5;
    let output: u32 = basic_service.call(input).await.unwrap();
    assert_eq!(output, 5);
    println!("Basic pattern test passed");

    // I/O pattern of a layer mediating protocol translation
    // It acts like a server layer would, accepting a read write pair,
    // and it acts like a client, sending a read write pair to the next layer
    let mut io_service = ServiceBuilder::new().layer(IoTowerLayer::default())
        .service(service_fn(io_service_fn));
    // We create a read/write pair that we can send into the tower layer, just like we have in the tower layer itself
    let (tower_read, mut this_write) = simplex(1024);
    let (mut this_read, tower_write) = simplex(1024);
    // We spawn the service, which will read from the read side and write to the write side
    let task = tokio::spawn(io_service.call((tower_read, tower_write)));
    // Lets send a string to the tower layer
    this_write.write_all(b"RapidRecast").await.unwrap();
    // And read the response
    let mut buffer = [0u8; 1024];
    let sz = this_read.read(&mut buffer).await.unwrap();
    let response = std::str::from_utf8(&buffer[..sz]).unwrap();
    assert_eq!(response, "!!!RapidRecast ,olleH");
    // We can now clean up (make sure everything terminates)
    drop(this_write);
    drop(this_read);
    task.await.unwrap().unwrap();
    println!("I/O pattern test passed");

    // Channel pattern
}

async fn basic_service_fn(input: u16) -> Result<u32, ()> {
    Ok(input as u32)
}

/// We have implemented this function using generic types, demonstrating that any downstream service
/// can adapt to this layer, providing they implement the necessary traits.
async fn io_service_fn<Reader: AsyncReadExt + Unpin, Writer: AsyncWriteExt + Unpin>((mut reader, mut writer): (Reader, Writer)) -> Result<(), ()> {
    let mut buffer = [0u8; 1024];
    let sz = reader.read(&mut buffer).await.unwrap();
    // We are going to write "Hello, {}!!!" as a response
    let mut return_data = Vec::new();
    return_data.extend_from_slice(b"Hello, ");
    return_data.extend_from_slice(&buffer[..sz]);
    return_data.extend_from_slice(b"!!!");
    writer.write_all(&return_data).await.unwrap();
    Ok(())
}
