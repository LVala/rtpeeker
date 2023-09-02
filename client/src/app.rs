mod gui;

#[derive(Default)]
pub struct App {
    gui: Option<gui::Gui>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.gui.is_none() {
            self.connect(ctx.clone(), frame);
        }

        if let Some(gui) = &mut self.gui {
            gui.ui(ctx);
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
                self.gui = Some(gui::Gui::new(ws_sender, ws_receiver));
            }
            Err(err) => {
                log::error!("Failed to connect to WebSocket: {}", err);
            }
        }
    }
}
