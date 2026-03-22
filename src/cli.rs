use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,

    #[arg(long, default_value = "mongodb")]
    pub(crate) protocol: String,

    #[arg(long, default_value = "localhost")]
    pub(crate) host: String,

    #[arg(long, default_value_t = 20667)]
    pub(crate) port: i32,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Get detailed information on each collection
    ListCollectionDetails {
        /// Database name
        db: String,
    },

    /// List the collections in a database
    ListCollections {
        /// Database name
        db: String,
    },

    /// List the databases
    ListDatabases,

    /// Estimate document count
    EstimateDocumentCount,

    /// Get an example document
    Example {
        /// Database name
        db: String,
        /// Collection name
        coll: String,
    },
}
