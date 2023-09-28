use std::ops::Div;

use eframe::egui::plot::{Line, Plot, PlotPoints};
use eframe::emath::Align;
use egui::{Color32, Layout, Vec2};
use egui_extras::{Column, TableBody, TableBuilder};

use crate::streams::RefStreams;

pub struct RtpStreamsTable {
    streams: RefStreams,
}

impl RtpStreamsTable {
    pub fn new(streams: RefStreams) -> Self {
        Self { streams }
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.build_table(ui);
        });
    }

    fn build_table(&mut self, ui: &mut egui::Ui) {
        let header_labels = [
            ("Stream alias", ""),
            ("SSRC", "RTP SSRC (Synchronization Source Identifier) identifies the source of an RTP stream"),
            ("Source", "Source IP address and port"),
            ("Destination", "Destination IP address and port"),
            ("Number of packets", "Number of packets in stream"),
            ("Duration", "Difference between last timestamp and first timestamp."),
            ("Lost packets", "Difference between last timestamp and first timestamp."),
            ("Jitter", ""),
            ("Jitter plot", ""),
        ];
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .stick_to_bottom(true)
            .columns(Column::remainder().at_least(40.0), 8)
            .column(Column::remainder().at_least(380.0).resizable(false))
            .header(30.0, |mut header| {
                header_labels.iter().for_each(|(label, desc)| {
                    header.col(|ui| {
                        ui.heading(label.to_string())
                            .on_hover_text(desc.to_string());
                    });
                });
            })
            .body(|body| {
                self.build_table_body(body);
            });
    }

    fn build_table_body(&mut self, body: TableBody) {
        let mut streams = self.streams.borrow_mut();
        let ssrcs: Vec<_> = streams.streams.keys().cloned().collect();

        body.rows(100.0, streams.streams.len(), |id, mut row| {
            let stream_ssrc = ssrcs.get(id).unwrap();
            let stream = streams.streams.get_mut(stream_ssrc).unwrap();

            row.col(|ui| {
                ui.with_layout(Layout::default().with_cross_align(Align::Min), |ui| {
                    let is_dark_mode = ui.style_mut().visuals.dark_mode;
                    ui.style_mut().visuals.extreme_bg_color = if id % 2 == 0 {
                        if is_dark_mode {
                            Color32::from_rgb(32, 32, 32)
                        } else {
                            Color32::from_rgb(255, 255, 255)
                        }
                    } else if is_dark_mode {
                        Color32::from_rgb(28, 28, 28)
                    } else {
                        Color32::from_rgb(247, 247, 247)
                    };
                    ui.text_edit_singleline(&mut stream.display_name);
                });
            });

            row.col(|ui| {
                ui.label(stream.ssrc.to_string());
            });
            row.col(|ui| {
                ui.label(stream.source_addr.to_string());
            });
            row.col(|ui| {
                ui.label(stream.destination_addr.to_string());
            });
            row.col(|ui| {
                ui.label(stream.rtp_packets.len().to_string());
            });
            row.col(|ui| {
                ui.label(format!("{:?}", stream.duration));
            });
            row.col(|ui| {
                ui.label(format!("{:.2}%", stream.lost_percentage));
            });
            row.col(|ui| {
                ui.label(format!("{:.10}", stream.jitter.to_string()));
            });
            row.col(|ui| {
                ui.vertical_centered_justified(|ui| {
                    let points: PlotPoints = (0..stream.jitter_history.len())
                        .map(|i| [i as f64, *stream.jitter_history.get(i).unwrap()])
                        .collect();

                    let zero_axis: PlotPoints = (0..(stream.jitter_history.len()
                        + (stream.jitter_history.len().div(5))))
                        .map(|i| [i as f64, 0.0])
                        .collect();

                    let line = Line::new(points).name("jitter");
                    let line_zero_axis = Line::new(zero_axis).color(Color32::GRAY);
                    Plot::new(id.to_string())
                        .show_background(false)
                        .show_axes([false, false])
                        .label_formatter(|name, value| {
                            if name.ne("jitter") || value.x.fract() != 0.0 {
                                return "".to_string();
                            }
                            format!("no = {}\njitter = {:.5}", value.x, value.y)
                        })
                        .set_margin_fraction(Vec2::new(0.1, 0.1))
                        .allow_scroll(false)
                        .show(ui, |plot_ui| {
                            plot_ui.line(line_zero_axis);
                            plot_ui.line(line);
                        });
                    ui.add_space(7.0);
                });
            });
        });
    }
}
