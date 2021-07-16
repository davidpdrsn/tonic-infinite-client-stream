use foo::foo_client::FooClient;
use foo::foo_server::{Foo, FooServer};
use futures::prelude::*;
use tonic::transport::Channel;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

pub mod foo {
    tonic::include_proto!("foo");
}

#[tokio::main]
async fn main() {
    match &*std::env::args().nth(1).expect("no command line args") {
        "client" => run_client().await,
        "server" => run_server().await,
        other => {
            panic!(
                "command line arg must be either `client` or `server`. Got: {}",
                other
            );
        }
    }
}

async fn run_server() {
    let addr = "127.0.0.1:50051".parse().unwrap();

    println!("FooServer listening on: {}", addr);

    Server::builder()
        .add_service(FooServer::new(FooService))
        .serve(addr)
        .await
        .unwrap();
}

#[derive(Debug)]
pub struct FooService;

#[tonic::async_trait]
impl Foo for FooService {
    async fn subscribe_stream(
        &self,
        request: Request<tonic::Streaming<()>>,
    ) -> Result<Response<()>, Status> {
        println!("client connected");

        let mut stream = request.into_inner();
        tokio::spawn(async move {
            while let Some(msg) = stream.next().await {
                if msg.is_err() {
                    break;
                }
            }

            println!("client disconnected");
        });

        Ok(Response::new(()))
    }
}

async fn run_client() {
    async fn send_request(client: &mut FooClient<Channel>) {
        println!("running client");

        let outbound = futures::stream::repeat(());
        let response = client
            .subscribe_stream(Request::new(outbound))
            .await
            .unwrap();

        println!("response = {:?}", response);
    }

    let mut client = FooClient::connect("http://127.0.0.1:50051").await.unwrap();
    send_request(&mut client).await;
    drop(client);

    futures::future::pending::<()>().await;
}
