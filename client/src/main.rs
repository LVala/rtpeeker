#[cfg(target_arch = "wasm32")]
mod app;

const CANVAS_ID: &str = "the_canvas_id";

#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                CANVAS_ID,
                web_options,
                Box::new(|_cc| Box::<app::App>::default()),
            )
            .await
            .expect("Error: failed to start eframe");
    });
}

// trick to be able to run tests in CI
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    panic!("Only wasm32 target supported");
}
