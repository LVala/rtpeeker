use eframe::egui;
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use log::{error, warn};
use packets_table::PacketsTable;
use rtpeeker_common::{Packet, Request};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

mod packets_table;

type Packets = Rc<RefCell<BTreeMap<usize, Packet>>>;

pub struct Gui {
    ws_sender: WsSender,
    ws_receiver: WsReceiver,
    is_capturing: bool,
    // some kind of sparse vector would be the best
    // but this will do
    packets: Packets,
    packets_table: PacketsTable,
}

impl Gui {
    pub fn new(ws_sender: WsSender, ws_receiver: WsReceiver) -> Self {
        let packets = Packets::default();
        let packets_table = PacketsTable::new(packets.clone());

        Self {
            ws_sender,
            ws_receiver,
            is_capturing: true,
            packets,
            packets_table,
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
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
                        self.packets.borrow_mut().clear();
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
                let _ = ui.button("📦 Packets");
            });
        });

        self.packets_table.ui(ctx);
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

            self.packets.borrow_mut().insert(packet.id, packet);
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
