use clap::{Parser, Subcommand};
use rtpeeker::cmd;

#[tokio::main]
async fn main() -> Result<(), ()> {
    let cli = RtpeekerArgs::parse();

    cli.run().await
}

#[derive(Debug, Parser)]
#[clap(author = "Łukasz Wala", version, about)]
struct RtpeekerArgs {
    #[clap(subcommand)]
    pub(crate) action: RtpeekerSubcommands,
}

impl RtpeekerArgs {
    pub async fn run(self) -> Result<(), ()> {
        match self.action {
            RtpeekerSubcommands::Run(inner) => inner.run().await,
            RtpeekerSubcommands::List(inner) => inner.run().await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum RtpeekerSubcommands {
    /// Run the app
    Run(cmd::run::Run),

    /// List network interfaces
    List(cmd::list::List),
}
