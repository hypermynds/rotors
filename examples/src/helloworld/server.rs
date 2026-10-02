use tonic::transport::Server;

use self::proto::{
    greeter_server::{Greeter, GreeterServer},
    HelloReply, HelloRequest,
};

mod proto;

#[derive(Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: tonic::Request<HelloRequest>,
    ) -> tonic::Result<tonic::Response<HelloReply>> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(tonic::Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let greeter = MyGreeter::default();

    println!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::greeter_client::GreeterClient;
    use std::time::Duration;
    use tokio::{net::TcpListener, sync::oneshot, time::timeout};
    use tokio_stream::wrappers::TcpListenerStream;

    #[tokio::test]
    async fn generated_client_and_server_round_trip() {
        timeout(Duration::from_secs(10), async {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let (shutdown_sender, shutdown_receiver) = oneshot::channel();
            let server = tokio::spawn(async move {
                Server::builder()
                    .add_service(GreeterServer::new(MyGreeter::default()))
                    .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                        let _ = shutdown_receiver.await;
                    })
                    .await
            });

            let mut client = GreeterClient::connect(format!("http://{address}"))
                .await
                .unwrap();
            let response = client
                .say_hello(HelloRequest {
                    name: "Rotors".into(),
                })
                .await;

            drop(client);
            shutdown_sender.send(()).unwrap();
            server.await.unwrap().unwrap();

            assert_eq!(response.unwrap().into_inner().message, "Hello Rotors!");
        })
        .await
        .expect("generated gRPC round trip timed out");
    }
}
