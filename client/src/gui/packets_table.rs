use super::Packets;

pub struct PacketsTable {
    packets: Packets,
}

impl PacketsTable {
    pub fn new(packets: Packets) -> Self {
        Self { packets }
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.build_table(ui);
        });
    }

    fn build_table(&self, ui: &mut egui::Ui) {}
}
