use crate::rtp_packets_table;
use eframe::egui;
use eframe::egui::{Context, Ui};

#[derive(Default)]
pub struct ViewState<'a> {
    rtp_packets_table: rtp_packets_table::RtpPacketsTable<'a>,
    is_rtp_packets_table_visible: bool,
    picked_path: Option<String>,
}

impl<'a> eframe::App for ViewState<'a> {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu bar").show(ctx, |ui| {

            // use egui::plot::{Line, Plot, PlotPoints};
            // let sin: PlotPoints = (0..1000).map(|i| {
            //     let x = i as f64 * 0.01;
            //     [x, x.sin()]
            // }).collect();
            // let line = Line::new(sin);
            // Plot::new("my_plot").view_aspect(2.0).show(ui, |plot_ui| plot_ui.line(line));

            ui.horizontal(|ui| {
                self.open_pcap_file_button(ui);
                self.show_rtp_packets_button(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |_| {
            self.show_or_hide_rtp_packets_window(ctx);
        });
    }
}

impl<'a> ViewState<'a> {
    fn open_pcap_file_button(&mut self, ui: &mut Ui) {
        if ui.button("Open pcap file").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                self.picked_path = Some(path.display().to_string());
            }
        }
    }

    fn show_rtp_packets_button(&mut self, ui: &mut Ui) {
        let table_button_text = if self.is_rtp_packets_table_visible {
            "Hide RTP packets"
        } else {
            "Show RTP packets"
        };
        if ui.button(table_button_text).clicked() {
            self.is_rtp_packets_table_visible = !self.is_rtp_packets_table_visible
        }
    }

    fn show_or_hide_rtp_packets_window(&mut self, ctx: &Context) {
        if self.is_rtp_packets_table_visible {
            self.rtp_packets_table.show(
                ctx,
                &mut self.is_rtp_packets_table_visible,
                &mut self.picked_path,
            )
        }
    }
}
