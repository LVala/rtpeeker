use crate::streams::{stream::Stream, RefStreams};
use egui::plot::{Line, Plot, PlotPoints};
use egui::{TextEdit, Vec2};
use egui_extras::{Column, TableBody, TableBuilder};
use ewebsock::{WsMessage, WsSender};
use rtpeeker_common::{Request, StreamKey};

const SDP_PROMPT: &str = "Paste your SDP media section here, e.g.
m=audio 5004 RTP/AVP 96
c=IN IP4 239.30.22.1
a=rtpmap:96 L24/48000/2
a=recvonly
";

pub struct RtpStreamsTable {
    streams: RefStreams,
    ws_sender: WsSender,
    sdp_window_open: bool,
    chosen_key: Option<StreamKey>,
    sdp: String,
}

impl RtpStreamsTable {
    pub fn new(streams: RefStreams, ws_sender: WsSender) -> Self {
        Self {
            streams,
            ws_sender,
            sdp_window_open: false,
            chosen_key: None,
            sdp: String::new(),
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.build_table(ui);
        });
        self.build_sdp_window(ctx);
    }

    fn build_sdp_window(&mut self, ctx: &egui::Context) {
        let Some((_, _, _, ssrc)) = self.chosen_key else {
            return;
        };

        let mut send_sdp = false;

        egui::Window::new(format!("SDP - {:x}", ssrc))
            .open(&mut self.sdp_window_open)
            .default_width(800.0)
            .default_height(800.0)
            .vscroll(true)
            .show(ctx, |ui| {
                TextEdit::multiline(&mut self.sdp)
                    .hint_text(SDP_PROMPT)
                    .desired_rows(30)
                    .desired_width(f32::INFINITY)
                    .show(ui);
                ui.add_space(10.0);
                if ui.button(format!("Set SDP for {:x}", ssrc)).clicked() {
                    send_sdp = true;
                }
            });

        if send_sdp {
            self.send_sdp_request();
            self.sdp_window_open = false;
        }
    }

    fn build_table(&mut self, ui: &mut egui::Ui) {
        let header_labels = [
            ("Alias", "Locally assigned SSRC alias to make differentiating streams more convenient"),
            ("SSRC", "RTP SSRC (Synchronization Source Identifier) identifies the source of an RTP stream"),
            ("Source", "Source IP address and port"),
            ("Destination", "Destination IP address and port"),
            ("CNAME", "Source Description CNAME value, if received"),
            ("Packet count", "Number of packets in stream"),
            ("Packet loss", "Percentage of packets lost"),
            ("Duration", "Difference between last timestamp and first timestamp."),
            ("Mean jitter", "Average of jitter (in ms) for all of the packets"),
            ("Mean bitrate", "Sum of packet sizes (IP header included) divided by stream's duration (in kbps)"),
            ("Mean packet rate", "Number of packets divided by stream's duration"),
            ("Jitter history", "Plot representing jitter for all of the stream's packets")
        ];
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .stick_to_bottom(true)
            .column(Column::initial(50.0).at_least(50.0))
            .column(Column::initial(80.0).at_least(80.0))
            .columns(Column::initial(140.0).at_least(140.0), 2)
            .columns(Column::initial(80.0).at_least(80.0), 3)
            .column(Column::initial(70.0).at_least(70.0))
            .columns(Column::initial(80.0).at_least(80.0), 2)
            .column(Column::initial(70.0).at_least(70.0))
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
        let keys: Vec<_> = streams.streams.keys().cloned().collect();

        body.rows(100.0, streams.streams.len(), |id, mut row| {
            let key = keys.get(id).unwrap();
            let stream = streams.streams.get_mut(key).unwrap();

            row.col(|ui| {
                let text_edit = TextEdit::singleline(&mut stream.alias).frame(false);
                ui.add(text_edit);
            });

            row.col(|ui| {
                ui.label(format!("{:x}", stream.ssrc));
            });
            row.col(|ui| {
                ui.label(stream.source_addr.to_string());
            });
            row.col(|ui| {
                ui.label(stream.destination_addr.to_string());
            });
            row.col(|ui| {
                ui.label(stream.cname.as_ref().unwrap_or(&"N/A".to_string()));
            });
            row.col(|ui| {
                ui.label(stream.rtp_packets.len().to_string());
            });
            row.col(|ui| {
                let lost = stream.get_expected_count() - stream.rtp_packets.len();
                let lost_fraction = lost as f64 / stream.get_expected_count() as f64;
                ui.label(format!("{:.3}%", lost_fraction * 100.0));
            });
            row.col(|ui| {
                let duration = stream.get_duration().as_secs_f64();
                ui.label(format!("{:.3}s", duration));
            });
            row.col(|ui| {
                let jitter = stream.get_mean_jitter() * 1000.0;
                ui.label(format!("{:.3}ms", jitter));
            });
            row.col(|ui| {
                let bitrate = stream.get_mean_bitrate() / 1000.0;
                ui.label(format!("{:.3}", bitrate));
            });
            row.col(|ui| {
                let packet_rate = stream.get_mean_packet_rate();
                ui.label(format!("{:.3}", packet_rate));
            });
            let (_, resp) = row.col(|ui| {
                build_jitter_plot(ui, stream);
            });

            resp.context_menu(|ui| {
                if ui.button("Set SDP").clicked() {
                    ui.close_menu();
                    self.chosen_key = Some(*key);
                    self.sdp = String::new();
                    self.sdp_window_open = true;
                }
            });
        });
    }

    fn send_sdp_request(&mut self) {
        let request = Request::ParseSdp(self.chosen_key.unwrap(), self.sdp.clone());

        let Ok(msg) = request.encode() else {
            log::error!("Failed to encode a request message");
            return;
        };
        let msg = WsMessage::Binary(msg);

        self.ws_sender.send(msg);
    }
}

fn build_jitter_plot(ui: &mut egui::Ui, stream: &Stream) {
    ui.vertical_centered_justified(|ui| {
        let points: PlotPoints = stream
            .rtp_packets
            .iter()
            .enumerate()
            .filter_map(|(ix, rtp)| rtp.jitter.map(|jitter| [ix as f64, jitter]))
            .collect();

        let line = Line::new(points).name("jitter");
        let key = format!(
            "{}{}{}{}",
            stream.ssrc, stream.source_addr, stream.destination_addr, stream.protocol
        );
        Plot::new(key)
            .show_background(false)
            .show_axes([true, true])
            .label_formatter(|_name, value| {
                format!("packet id: {}\njitter = {:.3}ms", value.x, value.y)
            })
            .set_margin_fraction(Vec2::new(0.1, 0.1))
            .allow_scroll(false)
            .show(ui, |plot_ui| {
                plot_ui.line(line);
            });
        ui.add_space(7.0);
    });
}
