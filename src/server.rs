use crate::sniffer::Sniffer;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt, TryFutureExt,
};
use log::{error, info, warn};
use rtpeeker_common::packet::SessionProtocol;
use rtpeeker_common::Source;
use rtpeeker_common::{Request, Response};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::{mpsc, mpsc::UnboundedSender, RwLock};
use warp::ws::{Message, WebSocket};
use warp::Filter;

const DIST_DIR: &str = "client/dist";
const WS_PATH: &str = "ws";
static NEXT_CLIENT_ID: AtomicUsize = AtomicUsize::new(1);

struct Client {
    pub sender: mpsc::UnboundedSender<Message>,
    pub source: Option<Source>,
}

impl Client {
    pub fn new(sender: mpsc::UnboundedSender<Message>) -> Self {
        Self {
            sender,
            source: None,
        }
    }
}

type Clients = Arc<RwLock<HashMap<usize, Client>>>;
type Packets = Arc<RwLock<Vec<Response>>>;
type PacketsMap = Arc<HashMap<Source, Packets>>;

pub async fn run(sniffers: HashMap<String, Sniffer>, addr: SocketAddr) {
    let clients = Clients::default();
    let mut source_to_packets = HashMap::new();

    // a bit of repetition, but Rust bested me this time
    for (_file, sniffer) in sniffers {
        let packets = Packets::default();
        source_to_packets.insert(sniffer.source.clone(), packets.clone());

        let cloned_clients = clients.clone();
        tokio::task::spawn(async move {
            sniff(sniffer, packets, cloned_clients).await;
        });
    }

    let source_to_packets = Arc::new(source_to_packets);

    let clients_filter = warp::any().map(move || clients.clone());
    let source_to_packets_filter = warp::any().map(move || source_to_packets.clone());
    let ws = warp::path(WS_PATH)
        .and(warp::ws())
        .and(clients_filter)
        .and(source_to_packets_filter)
        .map(|ws: warp::ws::Ws, clients_cl, source_to_packets_cl| {
            ws.on_upgrade(move |socket| client_connected(socket, clients_cl, source_to_packets_cl))
        });

    let routes = ws.or(warp::fs::dir(DIST_DIR));

    println!("RTPeeker running on http://{}/", addr);
    warp::serve(routes).try_bind(addr).await;
}

async fn client_connected(ws: WebSocket, clients: Clients, source_to_packets: PacketsMap) {
    let client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);

    info!("New client connected, assigned id: {}", client_id);

    let (mut ws_tx, ws_rx) = ws.split();

    send_pcap_filenames(&client_id, &mut ws_tx, &source_to_packets).await;

    let (tx, mut rx) = mpsc::unbounded_channel();

    tokio::task::spawn(async move {
        while let Some(message) = rx.recv().await {
            ws_tx
                .send(message)
                .unwrap_or_else(|e| {
                    error!("WebSocket `send` error: {}, client_id: {}", e, client_id);
                })
                .await;
        }
    });

    clients.write().await.insert(client_id, Client::new(tx));

    handle_messages(client_id, ws_rx, &clients, &source_to_packets).await;

    info!("Client disconnected, client_id: {}", client_id);
    clients.write().await.remove(&client_id);
}

async fn send_pcap_filenames(
    client_id: &usize,
    ws_tx: &mut SplitSink<WebSocket, Message>,
    source_to_packets: &Arc<HashMap<Source, Packets>>,
) {
    let sources = source_to_packets.keys().cloned().collect();
    let response = Response::Sources(sources);

    let Ok(encoded) = response.encode() else {
        error!("Failed to encode packet, client_id: {}", client_id);
        return;
    };

    let msg = Message::binary(encoded);
    ws_tx
        .send(msg)
        .unwrap_or_else(|e| {
            error!("WebSocket send error: {}, client_id: {}", e, client_id);
        })
        .await;
}

async fn sniff(mut sniffer: Sniffer, packets: Packets, clients: Clients) {
    while let Some(result) = sniffer.next_packet().await {
        match result {
            Ok(mut pack) => {
                pack.guess_payload();
                let response = Response::Packet(pack);

                let Ok(encoded) = response.encode() else {
                    error!("Sniffer: failed to encode packet");
                    continue;
                };
                let msg = Message::binary(encoded);
                for (_, client) in clients.read().await.iter() {
                    match client {
                        Client {
                            source: Some(source),
                            sender,
                        } if *source == sniffer.source => {
                            sender.send(msg.clone()).unwrap_or_else(|e| {
                                error!("Sniffer: error while sending packet: {}", e);
                            });
                        }
                        _ => {}
                    }
                }
                packets.write().await.push(response);
            }
            Err(err) => info!("Error when capturing a packet: {:?}", err),
        }
    }
}

async fn send_all_packets(
    client_id: usize,
    packets: &Packets,
    ws_tx: &mut UnboundedSender<Message>,
) {
    for pack in packets.read().await.iter() {
        let Ok(encoded) = pack.encode() else {
            error!("Failed to encode packet, client_id: {}", client_id);
            continue;
        };
        let msg = Message::binary(encoded);
        ws_tx.send(msg).unwrap_or_else(|e| {
            error!("WebSocket `feed` error: {}, client_id: {}", e, client_id);
        });
    }

    info!(
        "Sucesfully send already captured packets, client_id: {}",
        client_id
    );
}

async fn reparse_packet(
    client_id: usize,
    packets: &Packets,
    clients: &Clients,
    id: usize,
    cur_source: &Source,
    packet_type: SessionProtocol,
) {
    let mut packets = packets.write().await;
    let Some(response_packet) = packets.get_mut(id) else {
        warn!(
            "Received reparse request for non-existent packet {}, client_id: {}",
            id, client_id
        );
        return;
    };

    let Response::Packet(packet) = response_packet else {
        unreachable!("");
    };
    packet.parse_as(packet_type);

    let Ok(encoded) = response_packet.encode() else {
        error!("Failed to encode packet, client_id: {}", client_id);
        return;
    };
    let msg = Message::binary(encoded);
    for (_, client) in clients.read().await.iter() {
        match client {
            Client {
                source: Some(source),
                sender,
            } if *source == *cur_source => {
                sender.send(msg.clone()).unwrap_or_else(|e| {
                    error!("Sniffer: error while sending packet: {}", e);
                });
            }
            _ => {}
        };
    }
}

async fn handle_messages(
    client_id: usize,
    mut ws_rx: SplitStream<WebSocket>,
    clients: &Clients,
    packets: &PacketsMap,
) {
    // we could also simply pass the tx and source as function arguments
    // but it doesn't really matter
    let rd_clients = clients.read().await;
    let client = rd_clients.get(&client_id).unwrap();
    let mut source = client.source.clone();
    let mut sender = client.sender.clone();
    std::mem::drop(rd_clients);

    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                info!("Received message: {:?}, client_id: {}", msg, client_id);
                if !msg.is_binary() {
                    continue;
                }

                let msg = msg.into_bytes();
                let Ok(req) = Request::decode(&msg) else {
                    error!("Failed to decode request message");
                    continue;
                };

                match req {
                    Request::FetchAll => {
                        if let Some(ref cur_source) = source {
                            let packets = packets.get(cur_source).unwrap();
                            send_all_packets(client_id, packets, &mut sender).await;
                        }
                    }
                    Request::Reparse(id, packet_type) => {
                        // TODO: maybe the message should include the source?
                        // I see a potential for an RC
                        if let Some(ref cur_source) = source {
                            let packets = packets.get(cur_source).unwrap();
                            reparse_packet(
                                client_id,
                                packets,
                                clients,
                                id,
                                cur_source,
                                packet_type,
                            )
                            .await;
                        } else {
                            error!("Received reparse request from client without selected source, client_id: {}", client_id);
                        }
                    }
                    Request::ChangeSource(new_source) => {
                        let packets = packets.get(&new_source).unwrap();

                        source = Some(new_source);
                        let mut wr_clients = clients.write().await;
                        let client = wr_clients.get_mut(&client_id).unwrap();
                        client.source = source.clone();
                        std::mem::drop(wr_clients);

                        send_all_packets(client_id, packets, &mut sender).await;
                    }
                };
            }
            Err(e) => error!("WebSocket error: {}, client_id: {}", e, client_id),
        }
    }
}
