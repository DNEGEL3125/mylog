use clap::{command, Subcommand};

// "Path to the output file or directory. Errors if the path doesn't exist. Uses the file if it's a file; creates/uses a log file in the directory if it's a directory."

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Writes a message to the log file
    Write {
        /// The message to write
        #[arg(help = "The content of the message you want to write.")]
        message: Option<String>,

        /// Print more output
        #[arg(short, long, action = clap::ArgAction::SetTrue, help = "Enable verbose mode.")]
        verbose: bool,
    },

    /// Views the stored log messages
    View {
        /// Date to view
        #[arg(
            help = "The date of the logs to read in '%Y-%m-%d' format. If no date is provided, today's date will be used."
        )]
        date: Option<String>,

        /// Print more output
        #[arg(short, long, action = clap::ArgAction::SetTrue, help = "Enable verbose mode.")]
        verbose: bool,
    },

    /// Configure options
    Config {
        /// Configuration key (e.g., user.email)
        key: String,

        /// Configuration value (optional, e.g., xxx.com)
        value: Option<String>,
    },

    /// Edit logs
    Edit {
        /// Date to edit
        #[arg(
            help = "The date of the logs to read in '%Y-%m-%d' format. If no date is provided, today's date will be used."
        )]
        date: Option<String>,

        /// Print more output
        #[arg(short, long, action = clap::ArgAction::SetTrue, help = "Enable verbose mode.")]
        verbose: bool,
    },
}

#[derive(clap::Parser)]
#[command(name = "mylog")]
#[command(version = "1.0")]
#[command(about = "A logger tool for keeping a diary.", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
