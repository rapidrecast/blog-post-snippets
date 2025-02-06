use crate::helper::{DataTypeA, DataTypeB, IncrementingHandler};
use crate::pattern_basic::BasicPatternLayer;
use crate::pattern_chan::ChannelPatternLayer;
use crate::pattern_handler::{HandlerPatternLayer, ServiceHandler};
use crate::pattern_injected::InjectedPatternLayer;
use crate::pattern_io::IoPatternLayer;
use tokio::io::{simplex, AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tower::{service_fn, Service, ServiceBuilder};

mod pattern_io;
mod pattern_chan;
mod pattern_basic;
mod pattern_handler;
mod error;
mod helper;
mod pattern_injected;

#[tokio::main]
async fn main() {
    basic_example().await;
    io_example().await;
    channel_example().await;
    handler_example().await;
    injected_example().await;
}

async fn basic_example() {
    // Basic pattern of a layer accepting an input and giving an output
    // NOTE: this pattern is applied both to input and output.
    // In reality, it would be more likely to be applied to just one of them.
    // The other patterns would apply to the other side.
    let mut basic_service = ServiceBuilder::new().layer(BasicPatternLayer::<u8, u16, u32>::default()).service(service_fn(basic_service_fn));
    let input: u8 = 5;
    let output: u32 = basic_service.call(input).await.unwrap();
    assert_eq!(output, 5);
    println!("Basic pattern test passed");
}

async fn io_example() {
    // I/O pattern of a layer mediating protocol translation
    // It acts like a server layer would, accepting a read write pair,
    // and it acts like a client, sending a read write pair to the next layer
    let mut io_service = ServiceBuilder::new().layer(IoPatternLayer::default())
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
}

async fn channel_example() {
    // The channel pattern behaves similarly to the IO pattern, but it sends types between the layers
    // instead of bytes. This would tend to be the interface that users of the layer would be
    // exposed to - so they can send commands and receive clear responses
    let mut chan_service = ServiceBuilder::new()
        .layer(ChannelPatternLayer::new())
        .service(service_fn(chan_service_fn));

    // We will create the channel pair that we will send to the service.
    let (svc_sender, mut receiver) = channel::<DataTypeA>(1);
    let (mut sender, svc_receiver) = channel::<DataTypeA>(1);
    let task = tokio::spawn(chan_service.call((svc_receiver, svc_sender)));

    // We send an input
    sender.send(DataTypeA(5)).await.unwrap();

    // Receive the output
    let output = receiver.recv().await.unwrap();
    assert_eq!(output.0, 10);

    // Close the task
    task.await.unwrap().unwrap();
    println!("Channel pattern test passed");
}

async fn handler_example() {
    let mut handler_service = ServiceBuilder::new()
        .layer(HandlerPatternLayer::new())
        .service(service_fn(handler_service_fn));

    let mut our_handler = IncrementingHandler::new(654);
    handler_service.call(our_handler.clone()).await.unwrap();

    assert_eq!(our_handler.receive_message().await, 654 + 655);
    println!("Handler pattern test passed");
}

async fn injected_example() {
    // First we create our internal handler
    let mut internal_handler = IncrementingHandler::new(100);

    // Now we will create the tower stack, including our handler as part of the layer
    let mut injected_service = ServiceBuilder::new()
        .layer(InjectedPatternLayer::new(internal_handler.clone()))
        .service(service_fn(handle_injected_fn));

    // We invoke the tower call, which effectively would handle our entire protocol using the
    // internal handler
    injected_service.call(200).await.unwrap();

    let final_result = internal_handler.receive_message().await;
    assert_eq!(final_result, 200 * 2);
    println!("Injected pattern test passed");
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

async fn chan_service_fn((mut receiver, sender): (Receiver<DataTypeB>, Sender<DataTypeB>)) -> Result<(), ()> {
    let value = receiver.recv().await.unwrap();
    // Double the value and make sure it fits in a u8
    sender.send(DataTypeB((value.0 * 2) % 0xff)).await.unwrap();
    Ok(())
}

async fn handler_service_fn(_tower_input: ()) -> Result<IncrementingHandler, ()> {
    // We set the default value to 123, but we don't actually expect it to be used
    // It will be overwritten by the layer as a first step
    Ok(IncrementingHandler::new(123))
}

async fn handle_injected_fn(tower_input: u32) -> Result<u32, ()> {
    Ok(tower_input * 2)
}
