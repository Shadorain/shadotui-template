use better_panic::Settings;

use super::Tui;

pub fn initialize_panic_handler() {
    std::panic::set_hook(Box::new(|panic_info| {
        if let Ok(t) = Tui::new() {
            t.exit().unwrap();
        }
        Settings::auto()
            .most_recent_first(false)
            .lineno_suffix(true)
            .create_panic_handler()(panic_info);
        std::process::exit(1);
    }));
}

pub fn version() -> String {
    format!(
        "{}\n\nAuthors: {}",
        env!("CARGO_PKG_NAME"),
        clap::crate_authors!(),
    )
}
