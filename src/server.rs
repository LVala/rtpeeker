use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use futures_util::{stream::SplitStream, SinkExt, StreamExt, TryFutureExt};
use log::{error, info};
use tokio::sync::{mpsc, mpsc::UnboundedSender, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;

use rtpeeker_common::packet::PacketType;
use rtpeeker_common::{Packet, Request};

use crate::sniffer::Sniffer;

static DIST_DIR: &str = "client/dist";
static WS_PATH: &str = "ws";
static NEXT_CLIENT_ID: AtomicUsize = AtomicUsize::new(1);

type Clients = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;
type Packets = Arc<RwLock<Vec<Packet>>>;

pub async fn run<T: pcap::Activated + 'static>(sniffer: Sniffer<T>, addr: SocketAddr) {
    let packets = Packets::default();
    let clients = Clients::default();

    let clients_sniff = clients.clone();
    let packets_sniff = packets.clone();

    tokio::task::spawn(async move {
        sniff(sniffer, packets_sniff, clients_sniff).await;
    });

    let packets_filter = warp::any().map(move || packets.clone());
    let clients_filter = warp::any().map(move || clients.clone());
    let ws = warp::path(WS_PATH)
        .and(warp::ws())
        .and(packets_filter)
        .and(clients_filter)
        .map(|ws: warp::ws::Ws, packets, clients| {
            ws.on_upgrade(move |socket| client_connected(socket, packets, clients))
        });

    let routes = ws.or(warp::fs::dir(DIST_DIR));

    println!("RTPeeker running on http://{}/", addr);

    warp::serve(routes).run(addr).await;
}

async fn client_connected(ws: WebSocket, packets: Packets, clients: Clients) {
    let client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);

    info!("New client connected, assigned id: {}", client_id);

    let (mut ws_tx, ws_rx) = ws.split();

    // create channel to send incoming packets to client
    // and pass it to sniffer via shared state
    let (mut tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    send_all_packets(&packets, &mut tx, client_id).await;

    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            ws_tx
                .send(message)
                .unwrap_or_else(|e| {
                    error!("WebSocket `send` error: {}, client_id: {}", e, client_id);
                })
                .await;
        }
    });

    let clients_tx = tx.clone();
    clients.write().await.insert(client_id, clients_tx);

    handle_messages(ws_rx, tx, &packets, client_id).await;

    info!("Client disconnected, client_id: {}", client_id);
    clients.write().await.remove(&client_id);
}

async fn sniff<T: pcap::Activated>(mut sniffer: Sniffer<T>, packets: Packets, clients: Clients) {
    while let Some(result) = sniffer.next_packet() {
        match result {
            Ok(mut pack) => {
                pack.parse_as(PacketType::Rtp);
                let Ok(encoded) = pack.encode() else {
                    error!("Sniffer: failed to encode packet");
                    continue;
                };
                let msg = Message::binary(encoded);
                for (_, tx) in clients.read().await.iter() {
                    tx.send(msg.clone()).unwrap_or_else(|e| {
                        error!("Sniffer: error while sending packet: {}", e);
                    })
                }
                packets.write().await.push(pack);
            }
            Err(err) => error!("Error when capturing a packet: {:?}", err),
        }
    }
}

async fn send_all_packets(
    packets: &Packets,
    ws_tx: &mut UnboundedSender<Message>,
    client_id: usize,
) {
    for pack in packets.read().await.iter() {
        let Ok(encoded) = pack.encode() else {
            error!("Failed to encode packet, client_id: {}", client_id);
            continue;
        };
        let msg = Message::binary(encoded);
        ws_tx.send(msg).unwrap_or_else(|e| {
            error!("WebSocket `feed` error: {}, client_id: {}", e, client_id);
        })
    }

    info!(
        "Sucesfully send already captured packets, client_id: {}",
        client_id
    );
}

async fn handle_messages(
    mut ws_rx: SplitStream<WebSocket>,
    mut tx: UnboundedSender<Message>,
    packets: &Packets,
    client_id: usize,
) {
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                info!("Received message: {:?}, client_id: {}", msg, client_id);
                if msg.is_binary() {
                    let msg = msg.into_bytes();
                    let Ok(req) = Request::decode(&msg) else {
                        error!("Failed to decode request message");
                        continue;
                    };

                    match req {
                        Request::FetchAll => send_all_packets(packets, &mut tx, client_id).await,
                    };
                }
            }
            Err(e) => error!("WebSocket error: {}, client_id: {}", e, client_id),
        }
    }
}
