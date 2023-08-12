mod server;
mod sniffer;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    server::run("127.0.0.1:3550".parse().unwrap()).await;
}
