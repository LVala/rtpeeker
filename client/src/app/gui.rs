use std::collections::HashMap;
use std::fmt;

use eframe::egui;
use egui::{ComboBox, Ui};
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use log::{error, warn};
use rtpeeker_common::{Request, Response, Source};

use packets_table::PacketsTable;
use rtcp_packets_table::RtcpPacketsTable;
use rtp_packets_table::RtpPacketsTable;
use rtp_streams_table::RtpStreamsTable;

use crate::streams::RefStreams;
use rtp_streams_plot::RtpStreamsPlot;

mod packets_table;
mod rtcp_packets_table;
mod rtp_packets_table;
mod rtp_streams_plot;
mod rtp_streams_table;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    Packets,
    RtpPackets,
    RtcpPackets,
    Streams,
    Plot,
}

impl Tab {
    fn all() -> Vec<Self> {
        vec![
            Self::Packets,
            Self::RtpPackets,
            Self::RtcpPackets,
            Self::Streams,
            Self::Plot,
        ]
    }
}

impl fmt::Display for Tab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ret = match self {
            Self::Packets => "📦 All Packets",
            Self::RtpPackets => "🔈RTP Packets",
            Self::RtcpPackets => "📃 RTCP Packets",
            Self::Streams => "🔴 Streams",
            Self::Plot => "📈 Plot",
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
    streams: RefStreams,
    sources: Vec<Source>,
    selected_source: Option<Source>,
    tab: Tab,
    // would rather keep this in `Tab` enum
    // but it proved to be inconvinient
    packets_table: PacketsTable,
    rtp_packets_table: RtpPacketsTable,
    rtcp_packets_table: RtcpPacketsTable,
    rtp_streams_table: RtpStreamsTable,
    rtp_streams_plot: RtpStreamsPlot,
}

impl Gui {
    pub fn new(ws_sender: WsSender, ws_receiver: WsReceiver) -> Self {
        let streams = RefStreams::default();
        let packets_table = PacketsTable::new(streams.clone(), ws_sender.clone());
        let rtp_packets_table = RtpPacketsTable::new(streams.clone());
        let rtcp_packets_table = RtcpPacketsTable::new(streams.clone());
        let rtp_streams_table = RtpStreamsTable::new(streams.clone());
        let rtp_streams_plot = RtpStreamsPlot::new(streams.clone());

        Self {
            ws_sender,
            ws_receiver,
            is_capturing: true,
            streams,
            sources: Vec::new(),
            selected_source: None,
            tab: Tab::Packets,
            packets_table,
            rtp_packets_table,
            rtcp_packets_table,
            rtp_streams_table,
            rtp_streams_plot,
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
            Tab::RtcpPackets => self.rtcp_packets_table.ui(ctx),
            Tab::Streams => self.rtp_streams_table.ui(ctx),
            Tab::Plot => self.rtp_streams_plot.ui(ctx),
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
                        self.streams.borrow_mut().clear();
                    }

                    let button = side_button("↻");
                    let resp = ui
                        .add(button)
                        .on_hover_text("Refetch all previously captured packets");
                    if resp.clicked() {
                        self.streams.borrow_mut().clear();
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
                self.build_dropdown_source(ui);
                ui.separator();
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

    fn build_dropdown_source(&mut self, ui: &mut Ui) {
        let selected = match self.selected_source {
            Some(ref source) => source.to_string(),
            None => "Select packets source...".to_string(),
        };

        ComboBox::from_id_source("source_picker")
            .width(300.0)
            .wrap(false)
            .selected_text(selected)
            .show_ui(ui, |ui| {
                let mut was_changed = false;

                for source in self.sources.iter() {
                    let resp = ui.selectable_value(
                        &mut self.selected_source,
                        Some(source.clone()),
                        source.to_string(),
                    );
                    if resp.clicked() {
                        was_changed = true;
                    }
                }

                if was_changed {
                    self.streams.borrow_mut().clear();
                    self.change_source_request();
                }
            });
    }

    fn build_bottom_bar(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                let streams = self.streams.borrow();
                let count = streams.packets.id_count();
                let count_label = format!("Packets: {}", count);

                let captured_count = streams.packets.len();
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

            let Ok(response) = Response::decode(&msg) else {
                error!("Failed to decode request message");
                continue;
            };

            match response {
                Response::Packet(packet) => {
                    // this also adds the packet to self.packets
                    self.streams.borrow_mut().add_packet(packet);
                }
                Response::Sources(sources) => {
                    self.sources = sources;
                }
            }
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

    fn change_source_request(&mut self) {
        let selected = self.selected_source.as_ref().unwrap().clone();
        let request = Request::ChangeSource(selected);
        let Ok(msg) = request.encode() else {
            log::error!("Failed to encode a request message");
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

pub fn is_stream_visible(streams_visibility: &mut HashMap<u32, bool>, ssrc: u32) -> &mut bool {
    streams_visibility.entry(ssrc).or_insert(true)
}
