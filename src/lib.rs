mod cli;
mod commands;
mod connections;
mod errors;

use crate::{
    cli::{Cli, Commands, ConnectedCommands, StandaloneCommands},
    connections::Connection,
};
use clap::Parser;
use std::error::Error;

pub fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Connected(cmd) => {
            let conn = Connection::from_name(cli.name.as_deref())?;
            let client = conn.connect()?;
            match cmd {
                ConnectedCommands::EstimateDocumentCount { db, coll } => {
                    commands::estimate_document_count(&db, &coll, client)?
                }
                ConnectedCommands::Example { db, coll } => commands::example(&db, &coll, client)?,
                ConnectedCommands::ExampleFiltered { db, coll, filter } => {
                    commands::example_filtered(&db, &coll, &filter, client)?
                }
                ConnectedCommands::ListCollections { db } => {
                    commands::list_collections(&db, client)?
                }
                ConnectedCommands::ListCollectionDetails { db } => {
                    commands::list_collection_details(&db, client)?
                }
                ConnectedCommands::ListDatabases => commands::list_databases(client)?,
            }
        }
        Commands::Standalone(cmd) => match cmd {
            StandaloneCommands::CreateConnection(opts) => commands::create_connection(opts)?,
            StandaloneCommands::ListConnections => commands::list_connections()?,
        },
    }
    Ok(())
}
