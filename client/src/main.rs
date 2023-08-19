use rtpeeker_client::App;

static CANVAS_ID: &str = "the_canvas_id";

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
            .expect("failed to start eframe");
    });
}
