use clap::{Parser, Subcommand};

mod cmd;
mod server;
mod sniffer;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let cli = RtpeekerArgs::parse();
    cli.run().await;
}

#[derive(Debug, Parser)]
#[clap(version, about)]
struct RtpeekerArgs {
    #[clap(subcommand)]
    pub(crate) action: RtpeekerSubcommands,
}

impl RtpeekerArgs {
    pub async fn run(self) {
        match self.action {
            RtpeekerSubcommands::Run(inner) => inner.run().await,
            RtpeekerSubcommands::List(inner) => inner.run().await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum RtpeekerSubcommands {
    /// Run the app. E.g "run -f rtp.pcap webex.pcap -i etn0 wireless". Obtain help with "run --help"
    Run(cmd::run::Run),

    /// List network interfaces.
    List(cmd::list::List),
}
