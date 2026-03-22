pub mod cli;
pub mod commands;

use crate::cli::{Cli, Commands};
use clap::Parser;

pub fn main() -> mongodb::error::Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::EstimateDocumentCount { .. } => commands::estimate_document_count(&cli)?,
        Commands::Example { .. } => commands::example(&cli)?,
        Commands::ExampleFiltered { .. } => commands::example_filtered(&cli)?,
        Commands::ListCollectionDetails { .. } => commands::list_collection_details(&cli)?,
        Commands::ListCollections { .. } => commands::list_collections(&cli)?,
        Commands::ListDatabases => commands::list_databases(&cli)?,
    }
    Ok(())
}
