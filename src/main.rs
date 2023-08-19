use anyhow::Result;
use clap::Parser;
use tokio::sync::mpsc;
use tui::{initialize_panic_handler, version, App, Message};

mod tui;

// Define the command line arguments structure
#[derive(Parser, Debug)]
#[command(version = version(), about = "Shadotui template")]
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

    let (message_tx, mut message_rx) = mpsc::unbounded_channel::<Message>();

    let mut app = App::new(tick_rate).unwrap();
    tokio::spawn(async move {
        app.run(Some(message_tx)).await.unwrap();
    });

    loop {
        if let Some(message) = message_rx.recv().await {
            match message {
                Message::HelloWorld(s) => panic!("Got Message: {}", s),
                Message::Quit => break,
                _ => (),
            }
        }
    }
    Ok(())
}
