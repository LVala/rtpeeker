use eframe::egui;
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use log::{error, warn};
use rtpeeker_common::{Packet, Request};
use std::collections::HashMap;

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
        let host = Self::get_host(frame);
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

    #[cfg(target_arch = "wasm32")]
    fn get_host(frame: &eframe::Frame) -> String {
        frame.info().web_info.location.host
    }

    // ugly trick to allow running tests in CI on host target
    #[cfg(not(target_arch = "wasm32"))]
    fn get_host(frame: &eframe::Frame) -> String {
        "test_host".to_string()
    }
}

struct FrontEnd {
    ws_sender: WsSender,
    ws_receiver: WsReceiver,
    is_capturing: bool,
    // some kind of sparse vector would be the best
    // but this will do
    packets: HashMap<usize, Packet>,
}

impl FrontEnd {
    fn new(ws_sender: WsSender, ws_receiver: WsReceiver) -> Self {
        Self {
            ws_sender,
            ws_receiver,
            is_capturing: true,
            packets: HashMap::new(),
        }
    }

    fn ui(&mut self, ctx: &egui::Context) {
        if self.is_capturing {
            self.receive_packets()
        }

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = (0.0, 8.0).into();
        for (_text_style, font_id) in style.text_styles.iter_mut() {
            font_id.size = 20.0;
        }

        egui::SidePanel::left("side_panel")
            .resizable(false)
            .default_width(32.0)
            .show(ctx, |ui| {
                ui.set_style(style);
                ui.vertical_centered(|ui| {
                    // I'm struggling to add a margin...
                    ui.add_space(6.0);

                    let button = side_button("▶");
                    let resp = ui
                        .add_enabled(!self.is_capturing, button)
                        .on_hover_text("Resume packet capturing");
                    if resp.clicked() {
                        self.is_capturing = true
                    }

                    let button = side_button("⏸");
                    let resp = ui
                        .add_enabled(self.is_capturing, button)
                        .on_hover_text("Stop packet capturing");
                    if resp.clicked() {
                        self.is_capturing = false
                    }

                    let button = side_button("🗑");
                    let resp = ui
                        .add(button)
                        .on_hover_text("Discard previously captured packets");
                    if resp.clicked() {
                        self.packets.clear()
                    }

                    let button = side_button("↻");
                    let resp = ui
                        .add(button)
                        .on_hover_text("Refetch all previously captured packets");
                    if resp.clicked() {
                        self.refetch_packets()
                    }
                });

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(8.0);

                    egui::widgets::global_dark_light_mode_switch(ui);
                });
            });

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // TEMPORARY
                let _ = ui.button("♡ example button");
            });
        });

        // TEMPORARY
        egui::CentralPanel::default().show(ctx, |ui| {
            for (_id, packet) in self.packets.iter() {
                ui.label(format!("{:?}", packet));
            }
        });
    }

    fn receive_packets(&mut self) {
        while let Some(msg) = self.ws_receiver.try_recv() {
            let WsEvent::Message(msg) = msg else {
                warn!("Received special message: {:?}", msg);
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

            self.packets.insert(packet.id, packet);
        }
    }

    fn refetch_packets(&mut self) {
        let request = Request::FetchAll;
        let Ok(msg) = request.encode() else {
            error!("Failed to encode a request message");
            return;
        };
        let msg = WsMessage::Binary(msg);

        self.ws_sender.send(msg);
    }
}

fn side_button(text: &str) -> egui::Button {
    egui::Button::new(text)
        .min_size((30.0, 30.0).into())
        .rounding(egui::Rounding::same(9.0))
}
