use eframe::egui;
use eframe::egui::plot::{Line, Plot, PlotPoints};
use eframe::egui::Ui;
use egui::Window;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct JitterPlot<'a> {
    pub jitter_history: &'a Vec<(f64, f64)>,
}

impl<'a> JitterPlot<'a> {
    pub fn new(jitter_history: &'a Vec<(f64, f64)>) -> Self {
        Self { jitter_history }
    }
}

impl JitterPlot<'_> {
    pub fn show(&mut self, ctx: &egui::Context, mut open: bool, header: String) {
        Window::new(header)
            .open(&mut open)
            .resizable(true)
            .default_width(1200.0)
            .show(ctx, |ui| {
                self.ui(ui);
            });
    }

    fn ui(&mut self, ui: &mut Ui) {
        if self.jitter_history.is_empty() {
            ui.label("Jitter is not calculated, due to the fact that clock rate for payload type is undefined.");
        } else {
            ui.horizontal(|ui| {
                let points: PlotPoints = (0..self.jitter_history.len())
                    .map(|i| {
                        let jitter_entry = *self.jitter_history.get(i).unwrap();
                        [jitter_entry.1, jitter_entry.0]
                    })
                    .collect();
                let line = Line::new(points);
                Plot::new("")
                    .view_aspect(2.0)
                    .show(ui, |plot_ui| plot_ui.line(line));
            });
        }
    }
}
