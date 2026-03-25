mod cli;
mod commands;
mod connections;
mod errors;

use crate::{
    cli::{Cli, Commands},
    connections::Connection,
};
use clap::Parser;
use std::error::Error;

pub fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    if cli.command.requires_connection() {
        let conn = Connection::from_cli(&cli)?;
        let client = conn.connect()?;
        match &cli.command {
            Commands::EstimateDocumentCount { .. } => {
                commands::estimate_document_count(&cli, client)?
            }
            Commands::Example { .. } => commands::example(&cli, client)?,
            Commands::ExampleFiltered { .. } => commands::example_filtered(&cli, client)?,
            Commands::ListCollectionDetails { .. } => {
                commands::list_collection_details(&cli, client)?
            }
            Commands::ListCollections { .. } => commands::list_collections(&cli, client)?,
            Commands::ListDatabases => commands::list_databases(&cli, client)?,
            _ => {}
        }
    } else {
        match cli.command {
            Commands::CreateConnection(opts) => commands::create_connection(opts)?,
            Commands::ListConnections => commands::list_connections()?,
            _ => {}
        }
    }
    Ok(())
}
