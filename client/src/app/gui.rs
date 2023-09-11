use crate::app::gui::rtp_streams_table::RtpStreamsTable;
use eframe::egui;
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use log::{error, warn};
use packets_table::PacketsTable;
use rtp_packets_table::RtpPacketsTable;
use rtpeeker_common::{Packet, Request};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;
use std::rc::Rc;

mod packets_table;
mod rtp_packets_table;
mod rtp_streams_table;

type Packets = Rc<RefCell<BTreeMap<usize, Packet>>>;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    Packets,
    RtpPackets,
    RtpStreams,
}

impl Tab {
    fn all() -> Vec<Self> {
        vec![Self::Packets, Self::RtpPackets, Self::RtpStreams]
    }
}

impl fmt::Display for Tab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ret = match self {
            Self::Packets => "📦 Packets",
            Self::RtpPackets => "🔈RTP Packets",
            Self::RtpStreams => "🔴 RTP streams",
        };

        write!(f, "{}", ret)
    }
}

pub struct Gui {
    ws_sender: WsSender,
    ws_receiver: WsReceiver,
    is_capturing: bool,
    // some kind of sparse vector would be the best
    // but this will do
    packets: Packets,
    tab: Tab,
    // would rather keep this in `Tab` enum
    // but it proved to be inconvinient
    packets_table: PacketsTable,
    rtp_packets_table: RtpPacketsTable,
    rtp_streams_table: RtpStreamsTable,
}

impl Gui {
    pub fn new(ws_sender: WsSender, ws_receiver: WsReceiver) -> Self {
        let packets = Packets::default();
        let packets_table = PacketsTable::new(packets.clone(), ws_sender.clone());
        let rtp_packets_table = RtpPacketsTable::new(packets.clone());
        let rtp_streams_table = RtpStreamsTable::new(packets.clone());

        Self {
            ws_sender,
            ws_receiver,
            is_capturing: true,
            packets,
            tab: Tab::Packets,
            packets_table,
            rtp_packets_table,
            rtp_streams_table,
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        if self.is_capturing {
            self.receive_packets()
        }

        self.build_side_panel(ctx);
        self.build_top_bar(ctx);
        self.build_bottom_bar(ctx);

        match self.tab {
            Tab::Packets => self.packets_table.ui(ctx),
            Tab::RtpPackets => self.rtp_packets_table.ui(ctx),
            Tab::RtpStreams => self.rtp_streams_table.ui(ctx),
        };
    }

    fn build_side_panel(&mut self, ctx: &egui::Context) {
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
    }

    fn build_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                Tab::all().iter().for_each(|tab| {
                    if ui
                        .selectable_label(*tab == self.tab, tab.to_string())
                        .clicked()
                    {
                        self.tab = *tab;
                    }
                });
            });
        });
    }

    fn build_bottom_bar(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                let packets = self.packets.borrow();
                let count = match packets.last_key_value() {
                    Some((id, _)) => id + 1,
                    None => 0,
                };
                let count_label = format!("Packets: {}", count);

                let captured_count = packets.len();
                let captured_label = format!("Captured: {}", captured_count);

                let filtered_count = 0; // TODO
                let filtered_label = format!("Filtered: {}", filtered_count);
                let label = format!("{} • {} • {}", count_label, captured_label, filtered_label);
                ui.label(label);
            });
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
