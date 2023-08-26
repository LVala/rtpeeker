use rtpeeker_client::gui;

const CANVAS_ID: &str = "the_canvas_id";

fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                CANVAS_ID,
                web_options,
                Box::new(|_cc| Box::<App>::default()),
            )
            .await
            .expect("Error: failed to start eframe");
    });
}

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
