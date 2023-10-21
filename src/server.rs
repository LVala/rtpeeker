use futures_util::stream::SplitSink;
use futures_util::{stream::SplitStream, SinkExt, StreamExt, TryFutureExt};
use log::{error, info, warn};
use pcap::{Active, Offline};
use rtpeeker_common::packet::SessionProtocol;
use rtpeeker_common::{Request, Response};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::{mpsc, mpsc::UnboundedSender, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;

use crate::sniffer::Sniffer;

const DIST_DIR: &str = "client/dist";
const WS_PATH: &str = "ws";
static NEXT_CLIENT_ID: AtomicUsize = AtomicUsize::new(1);

type Clients = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;
type ClientsSource = Arc<RwLock<HashMap<usize, String>>>;
type Packets = Arc<RwLock<Vec<Response>>>;

pub async fn run(
    default_source: String,
    interface_sniffers: HashMap<String, Sniffer<Active>>,
    file_sniffers: HashMap<String, Sniffer<Offline>>,
    addr: SocketAddr,
) {
    let clients = Clients::default();
    let clients_source = ClientsSource::default();

    let mut source_to_packets_map: HashMap<String, Packets> = HashMap::new();
    for file_name in file_sniffers.keys() {
        source_to_packets_map.insert(file_name.clone(), Packets::default());
    }
    for interface_name in interface_sniffers.keys() {
        source_to_packets_map.insert(interface_name.clone(), Packets::default());
    }

    for (source, sniffer) in interface_sniffers {
        let source_to_packets_cloned = source_to_packets_map.get(&source).unwrap().clone();
        let client_source_cloned = clients_source.clone();
        let clients_cloned = clients.clone();
        tokio::task::spawn(async move {
            sniff(
                sniffer,
                source_to_packets_cloned,
                clients_cloned,
                client_source_cloned,
            )
            .await;
        });
    }

    for (source, file_sniffer) in file_sniffers {
        let source_to_packets_cloned = source_to_packets_map.get(&source).unwrap().clone();
        let client_source_cloned = clients_source.clone();
        let clients_cloned = clients.clone();
        tokio::task::spawn(async move {
            sniff(
                file_sniffer,
                source_to_packets_cloned,
                clients_cloned,
                client_source_cloned,
            )
            .await;
        });
    }

    let default_source_filter = warp::any().map(move || default_source.clone());
    let source_to_packets_filter = warp::any().map(move || source_to_packets_map.clone());
    let clients_filter = warp::any().map(move || clients.clone());
    let clients_source_filter = warp::any().map(move || clients_source.clone());
    let ws = warp::path(WS_PATH)
        .and(warp::ws())
        .and(default_source_filter)
        .and(clients_source_filter)
        .and(source_to_packets_filter)
        .and(clients_filter)
        .map(
            |ws: warp::ws::Ws,
             default_source,
             my_clients_source,
             file_sniffer_packets_filter,
             clients| {
                ws.on_upgrade(move |socket| {
                    client_connected(
                        socket,
                        default_source,
                        file_sniffer_packets_filter,
                        clients,
                        my_clients_source,
                    )
                })
            },
        );

    let routes = ws.or(warp::fs::dir(DIST_DIR));
    println!("RTPeeker running on http://{}/", addr);

    warp::serve(routes).run(addr).await;
}

async fn client_connected(
    ws: WebSocket,
    default_source: String,
    source_to_packets_map: HashMap<String, Packets>,
    clients: Clients,
    clients_source: ClientsSource,
) {
    let client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);

    info!("New client connected, assigned id: {}", client_id);

    let (mut ws_tx, ws_rx) = ws.split();

    // create channel to send incoming packets to client
    // and pass it to sniffer via shared state
    let (mut tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    clients_source
        .write()
        .await
        .insert(client_id, default_source.clone());

    send_pcap_filenames(
        &client_id,
        &mut ws_tx,
        &source_to_packets_map,
        default_source.clone(),
    )
    .await;

    let clients_source_read = clients_source.read().await;
    let source = clients_source_read.get(&client_id).unwrap();
    let packets = source_to_packets_map.get(source).unwrap();
    send_all_packets(packets, &mut tx, client_id).await;

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

    handle_messages(
        ws_rx,
        tx,
        &source_to_packets_map,
        &clients,
        client_id,
        &mut clients_source.clone(),
    )
    .await;

    info!("Client disconnected, client_id: {}", client_id);
    clients.write().await.remove(&client_id);
}

async fn send_pcap_filenames(
    client_id: &usize,
    ws_tx: &mut SplitSink<WebSocket, Message>,
    source_to_packets: &HashMap<String, Packets>,
    default_source: String,
) {
    let mut sources = Vec::new();
    for source in source_to_packets.keys() {
        sources.push(source.clone())
    }

    let response = Response::PcapExamples((sources, default_source));
    let Ok(encoded) = response.encode() else {
        error!("Failed to encode packet, client_id: {}", client_id);
        return;
    };

    let msg = Message::binary(encoded);
    ws_tx
        .send(msg)
        .unwrap_or_else(|e| {
            error!("WebSocket `feed` error: {}, client_id: {}", e, client_id);
        })
        .await;
}

async fn sniff<T: pcap::Activated>(
    mut sniffer: Sniffer<T>,
    packets: Packets,
    clients: Clients,
    clients_source: ClientsSource,
) {
    while let Some(result) = sniffer.next_packet() {
        match result {
            Ok(mut pack) => {
                pack.guess_payload();
                let Ok(encoded) = pack.encode() else {
                    error!("Sniffer: failed to encode packet");
                    continue;
                };
                let msg = Message::binary(encoded);
                for (_, tx) in clients.read().await.iter() {
                    for (_, source) in clients_source.read().await.iter() {
                        if sniffer.source == *source {
                            tx.send(msg.clone()).unwrap_or_else(|e| {
                                error!("Sniffer: error while sending packet: {}", e);
                            })
                        }
                    }
                }
                packets.write().await.push(Response::Packet(pack));
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

async fn reparse_packet(
    packets: &Packets,
    clients: &Clients,
    client_id: usize,
    id: usize,
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
    for (_, tx) in clients.read().await.iter() {
        tx.send(msg.clone()).unwrap_or_else(|e| {
            error!("Sniffer: error while sending packet: {}", e);
        })
    }
}

async fn handle_messages(
    mut ws_rx: SplitStream<WebSocket>,
    mut tx: UnboundedSender<Message>,
    all_packets: &HashMap<String, Packets>,
    clients: &Clients,
    client_id: usize,
    clients_sources: &mut ClientsSource,
) {
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
                        let client_sources_awaited = clients_sources.read().await;
                        let source = client_sources_awaited.get(&client_id).unwrap();
                        let packets = all_packets.get(source).unwrap();
                        send_all_packets(packets, &mut tx, client_id).await;
                    }
                    Request::Reparse(id, packet_type) => {
                        let client_sources_awaited = clients_sources.read().await;
                        let source = client_sources_awaited.get(&client_id).unwrap();
                        let packets = all_packets.get(source).unwrap();
                        reparse_packet(packets, clients, client_id, id, packet_type).await;
                    }
                    Request::ChangeSource(source) => {
                        // TODO after this line nothing happens, it blocks forever
                        // however it is necessary if we want reparse and fetch all
                        // work with selected source, not default one

                        // clients_sources.write().await.insert(client_id, source.clone());
                        let sniffer_packets = all_packets.get(&source).unwrap();
                        send_all_packets(sniffer_packets, &mut tx, client_id).await;
                    }
                };
            }
            Err(e) => error!("WebSocket error: {}, client_id: {}", e, client_id),
        }
    }
}
