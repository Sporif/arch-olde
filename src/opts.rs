pub use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};

/// A tool to show outdated packages in your Arch Linux system according to
/// the repology.org database.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Opts {
    /// Enable extra verbosity to report unexpected events,
    /// fetch progress and so on.
    #[command(flatten)]
    pub(crate) verbose: Verbosity<InfoLevel>,

    /// Use the full repology repo (instead of adding '&outdated=1'
    /// to the fetch url).
    #[arg(long)]
    pub(crate) full_repo: bool,
}
