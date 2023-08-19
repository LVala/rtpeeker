use eframe::egui;
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use log::{error, warn};
use rtpeeker_common::Packet;

#[derive(Default)]
pub struct App {
    is_connected: bool,
    frontend: Option<FrontEnd>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if !self.is_connected {
            self.connect(ctx.clone(), frame);
        }

        if let Some(frontend) = &mut self.frontend {
            frontend.ui(ctx);
        }
    }
}

impl App {
    fn connect(&mut self, ctx: egui::Context, frame: &eframe::Frame) {
        let host = frame.info().web_info.location.host;
        let uri = format!("ws://{}/ws", host);

        let wakeup = move || ctx.request_repaint(); // wake up UI thread on new message
        match ewebsock::connect_with_wakeup(uri, wakeup) {
            Ok((ws_sender, ws_receiver)) => {
                self.frontend = Some(FrontEnd::new(ws_sender, ws_receiver));
                self.is_connected = true;
            }
            Err(err) => {
                error!("Failed to connect to WebSocket: {}", err);
            }
        }
    }
}

struct FrontEnd {
    // ws_sender: WsSender,
    ws_receiver: WsReceiver,
    packets: Vec<Packet>,
}

impl FrontEnd {
    fn new(_ws_sender: WsSender, ws_receiver: WsReceiver) -> Self {
        Self {
            // ws_sender,
            ws_receiver,
            packets: Vec::new(),
        }
    }

    fn ui(&mut self, _ctx: &egui::Context) {
        self.receive_packets();
    }

    fn receive_packets(&mut self) {
        while let Some(msg) = self.ws_receiver.try_recv() {
            let WsEvent::Message(msg) = msg else {
                warn!("Received unexpected message: {:?}", msg);
                continue;
            };

            let WsMessage::Binary(msg) = msg else {
                warn!("Received unexpected message: {:?}", msg);
                continue;
            };

            let Ok(packet) = Packet::decode(&msg) else {
                warn!("Failed to decode message: {:?}", msg);
                continue;
            };

            self.packets.push(packet);
        }
    }
}
