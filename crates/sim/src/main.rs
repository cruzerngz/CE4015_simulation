#![allow(unused)]

mod traits;
use futures::future::{self, Ready};
use tarpc::{
    client, context,
    server::{self, Channel},
};

#[tarpc::service]
trait SomeRemoteMethods {
    async fn whats_up(name: String) -> String;
    async fn whats_there(which: bool) -> String;
}

#[derive(Clone)]
struct FunServer;

impl SomeRemoteMethods for FunServer {
    async fn whats_up(self, context: ::tarpc::context::Context, name: String) -> String {
        "asdasd".to_string()
    }

    async fn whats_there(self, context: ::tarpc::context::Context, which: bool) -> String {
        "Asdasddasdas".to_string()
    }
}

#[tarpc::service]
trait World {
    /// Returns a greeting for name.
    async fn hello(name: String) -> String;
}

// This is the type that implements the generated World trait. It is the business logic
// and is used to start the server.
#[derive(Clone)]
struct HelloServer;

impl World for HelloServer {
    async fn hello(self, _: context::Context, name: String) -> String {
        format!("Hello, {name}!")
    }
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let (client_transport, server_transport) = tarpc::transport::channel::unbounded();
    let server = tarpc::server::BaseChannel::with_defaults(server_transport);

    tokio::spawn(async { server.execute(FunServer.serve()) });

    // let client = SomeRemoteMethodsClient::new(Default::default(), client_transport).spawn();
    let client = SomeRemoteMethodsClient::new(client::Config::default(), client_transport).spawn();

    let x = client.whats_up(context::current(), false.to_string()).await;




}
