use anyhow::Result;
use clap::Parser;
use tui::{
    App,
    initialize_panic_handler, version,
};

mod tui;

// Define the command line arguments structure
#[derive(Parser, Debug)]
#[command(version = version(), about = "Ratatui template")]
struct Args {
    /// App tick rate
    #[arg(short, long, default_value_t = 1000)]
    app_tick_rate: u64,
    /// Render tick rate
    #[arg(short, long, default_value_t = 50)]
    render_tick_rate: u64,
}

// Main function
#[tokio::main]
async fn main() -> Result<()> {
    initialize_panic_handler();

    let args = Args::parse();
    let tick_rate = (args.app_tick_rate, args.render_tick_rate);

    let mut app = App::new(tick_rate)?;
    app.run().await?;

    Ok(())
}
