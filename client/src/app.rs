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
    is_capturing: bool,
    packets: Vec<Packet>,
}

impl FrontEnd {
    fn new(_ws_sender: WsSender, ws_receiver: WsReceiver) -> Self {
        Self {
            // ws_sender,
            ws_receiver,
            is_capturing: true,
            packets: Vec::new(),
        }
    }

    fn ui(&mut self, ctx: &egui::Context) {
        if self.is_capturing {
            self.receive_packets()
        }

        let mut style = (*ctx.style()).clone();
        for (_text_style, font_id) in style.text_styles.iter_mut() {
            font_id.size = 20.0;
        }

        egui::SidePanel::left("side_panel")
            .resizable(false)
            .default_width(32.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.set_style(style);

                    let button = side_button("▶");
                    let resp = ui.add(button).on_hover_text("Resume packet capturing");
                    if resp.clicked() {
                        self.is_capturing = true
                    }

                    let button = side_button("⏸");
                    let resp = ui.add(button).on_hover_text("Stop packet capturing");
                    if resp.clicked() {
                        self.is_capturing = false
                    }

                    let button = side_button("🗑");
                    ui.add(button)
                        .on_hover_text("Discard previously captured packets");
                    if resp.clicked() {
                        self.packets.clear()
                    }

                    let button = side_button("↻");
                    ui.add(button)
                        .on_hover_text("Refetch all previously captured packets");
                    if resp.clicked() {
                        self.refetch_packets()
                    }
                });
            });

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                let _ = ui.button("♡ example button");
            });
        });
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

    fn refetch_packets(&self) {
        todo!();
    }
}

fn side_button(text: &str) -> egui::Button {
    egui::Button::new(text)
        .min_size((30.0, 30.0).into())
        .rounding(egui::Rounding::same(9.0))
}
