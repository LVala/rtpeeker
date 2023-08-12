pub(crate) fn show_pick_file_dialog() -> String {
    loop {
        if let Some(path_buf) = rfd::FileDialog::new().pick_file() {
            if let Some(file_path) = path_buf.to_str() {
                return file_path.to_string();
            } else {
                println!("Invalid file path. Please try again.");
            }
        } else {
            println!("Error showing file dialog. Please try again.");
        }
    }
}
