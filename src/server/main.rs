use bytes::Bytes;
use http::Request;
use std::error::Error;
use tokio::net::TcpStream;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let _ = env_logger::try_init();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:5928").await?;

    loop {
        if let Ok((socket, _peer_addr)) = listener.accept().await {
            println!("accept connection");
            tokio::spawn(async move {
                if let Err(e) = serve(socket).await {
                    println!("serve error: {:?}", e);
                }
            });
        }
    }
}

async fn serve(socket: TcpStream) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut connection = h2::server::handshake(socket).await?;

    while let Some(result) = connection.accept().await {
        println!("accept stream");
        let (request, respond) = result?;
        println!("got request");
        tokio::spawn(async move {
            if let Err(e) = handle_request(request, respond).await {
                println!("error while handling request: {}", e);
            }
        });
    }

    println!("Connection CLOSE");
    Ok(())
}

async fn handle_request(
    mut request: Request<h2::RecvStream>,
    mut respond: h2::server::SendResponse<Bytes>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("got request: {:?}", request);

    tokio::spawn(async move {
        let body = request.body_mut();
        while let Some(data) = body.data().await {
            let chunk = data.unwrap();
            println!("got chunk: {:?}", chunk);
        }
    });

    tokio::spawn(async move {
        let response = http::Response::new(());
        let mut send = respond.send_response(response, false).unwrap();

        loop {
            send.send_data("server data".into(), false).unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    });

    Ok(())
}
