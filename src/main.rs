#[tokio::main]
async fn main() {
    warp::serve(warp::fs::dir("client/dist"))
        .run(([127, 0, 0, 1], 3550))
        .await;
}
