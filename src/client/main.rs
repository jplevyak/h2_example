use h2::client;
use http::{Method, Request};
use std::error::Error;
use tokio::net::TcpStream;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let _ = env_logger::try_init();

    let tcp = TcpStream::connect("127.0.0.1:5928").await?;
    let (mut client, h2) = client::handshake(tcp).await?;

    // Spawn a task to run the conn...
    tokio::spawn(async move {
        if let Err(e) = h2.await {
            println!("got error: {:?}", e);
        }
    });

    println!("sending request");

    let request = Request::builder()
        .method(Method::GET)
        .uri("https://localhost/")
        .body(())
        .unwrap();

    let (response, mut stream) = client.send_request(request, false).unwrap();

    println!("sent request");

    let response = response.await.unwrap();
    println!("got response: {:?}", response);

    // Get the body
    let mut body = response.into_body();

    tokio::spawn(async move {
        loop {
            stream
                .send_data("client data".into(), false)
                .expect("send_data error");
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    });

    tokio::spawn(async move {
        while let Some(chunk) = body.data().await {
            println!("got chunk: {:?}", chunk.unwrap());
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

    Ok(())
}
