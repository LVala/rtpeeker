use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(version, about)]
pub(crate) struct RtpeekerArgs {
    #[clap(subcommand)]
    pub(crate) action: CommandType,
}

#[derive(Debug, Subcommand)]
pub(crate) enum CommandType {
    /// Analyze packets from file
    File(FileSubcommand),

    /// Capture packets from network interface
    Interface(InterfaceSubcommand),

    /// List available devices, consider using flags to filter
    ListInterfaces(ListInterfaceSubcommand),
}

#[derive(Debug, Args)]
pub(crate) struct FileSubcommand {
    /// Relative path to pcap file
    pub(crate) file_path: String,

    /// Name of the socket to serve
    pub(crate) socket_to_serve: String,
}

#[derive(Debug, Args)]
pub(crate) struct InterfaceSubcommand {
    /// Interface name on which captures packets
    pub(crate) interface_name: String,

    /// Socket to serve packets. Expected format 168.192.1.3:422 or [2001:db8::1]:8080
    pub(crate) socket_to_serve: String,
}

#[derive(Debug, Args)]
pub(crate) struct ListInterfaceSubcommand {
    /// Filter network interfaces that are UP
    pub(crate) filter_up: Option<bool>,

    /// Filter network interfaces that are LOOPBACK
    pub(crate) filter_loopback: Option<bool>,

    /// Filter network interfaces that are RUNNING
    pub(crate) filter_running: Option<bool>,

    /// Filter network interfaces that are WIRELESS
    pub(crate) filter_wireless: Option<bool>,
}
