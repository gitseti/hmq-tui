use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[clap(disable_help_flag = true)]
pub struct Cli {
    #[arg(
        long,
        action = clap::ArgAction::HelpLong
    )]
    help: Option<bool>,

    #[arg(
        long,
        value_name = "FLOAT",
        help = "Tick rate, i.e. number of ticks per second",
        default_value_t = 1.0,
        hide = true
    )]
    pub tick_rate: f64,

    #[arg(
        long,
        value_name = "FLOAT",
        help = "Frame rate, i.e. number of frames per second",
        default_value_t = 30.0,
        hide = true
    )]
    pub frame_rate: f64,

    #[arg(
        long,
        help = "Activates debug mode which shows additional components for debugging purposes",
        hide = true
    )]
    pub debug: bool,

    #[arg(
        short,
        long,
        value_name = "STRING",
        help = "The HiveMQ hostname",
        default_value = "http://localhost"
    )]
    pub host: String,

    #[arg(
        short,
        long,
        value_name = "INTEGER",
        help = "The port of the HiveMQ Rest API",
        default_value_t = 8888
    )]
    pub port: usize,
}
