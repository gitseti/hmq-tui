#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use clap::Parser;
use color_eyre::eyre::Result;
use hmq_tui::app::App;
use hmq_tui::cli::Cli;
use hmq_tui::utils::{initialize_logging, initialize_panic_handler};

async fn tokio_main() -> Result<()> {
    initialize_logging()?;

    initialize_panic_handler()?;

    let args = Cli::parse();
    let hivemq_address = args.host + ":" + args.port.to_string().as_str();
    let mut app = App::new(args.tick_rate, args.frame_rate, hivemq_address, args.debug)?;
    app.run().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = tokio_main().await {
        eprintln!("{} error: Something went wrong", env!("CARGO_PKG_NAME"));
        Err(e)
    } else {
        Ok(())
    }
}
