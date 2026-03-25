use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,

    /// Name of the connection to use (see list-connections command)
    #[arg(short, long)]
    pub(crate) name: Option<String>,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Create a new connection
    CreateConnection(CreateConnectionArgs),

    /// Returns the document count in the metadata for the collection
    EstimateDocumentCount {
        /// Database name
        db: String,
        /// Collection name
        coll: String,
    },

    /// Get an example document
    Example {
        /// Database name
        db: String,
        /// Collection name
        coll: String,
    },

    /// Get an example document after filtering
    ExampleFiltered {
        /// Database name
        db: String,
        /// Collection name
        coll: String,
        /// Filter string
        filter: String,
    },

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

    /// List connections
    ListConnections,

    /// List the databases
    ListDatabases,
}

impl Commands {
    pub(crate) fn requires_connection(&self) -> bool {
        match self {
            Commands::ListDatabases => true,
            Commands::Example { .. } => true,
            Commands::ExampleFiltered { .. } => true,
            Commands::ListCollections { .. } => true,
            Commands::EstimateDocumentCount { .. } => true,
            Commands::ListCollectionDetails { .. } => true,
            _ => false,
        }
    }
}

#[derive(Args)]
pub(crate) struct CreateConnectionArgs {
    /// Connection name
    pub(crate) name: String,
    /// Hostname
    #[arg(long)]
    pub(crate) host: String,
    /// Port
    #[arg(long)]
    pub(crate) port: i32,
    /// Protocol
    #[arg(long, default_value = "mongodb")]
    pub(crate) protocol: String,
    /// Set as default connection
    #[arg(long)]
    pub(crate) default: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
