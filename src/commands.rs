use crate::cli::{Cli, Commands, CreateConnectionArgs};
use crate::connections::{read_connections, write_connections, Connection};
use crate::errors::ConnectionError;
use mongodb::{
    bson::{doc, to_document, Document},
    sync::{Client, Collection, Database},
};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use tabled::{
    settings::{object::Rows, Alignment, Settings, Style},
    Table, Tabled,
};

#[derive(Tabled)]
struct DetailRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Doc Count")]
    count: u64,
    #[tabled(rename = "Size")]
    size: String,
    #[tabled(rename = "Storage")]
    storage_size: String,
}

impl DetailRow {
    fn new(db: &Database, name: &str) -> Self {
        let cmd = doc! { "collStats": name };
        let coll: Collection<Document> = db.collection(name);
        let stats = db
            .run_command(cmd)
            .run()
            .expect("Could not get collection stats");
        let storage_size = stats
            .get_i32("storageSize")
            .expect("Could not get storage size");
        let size = stats.get_i32("size").expect("Could not get size");
        Self {
            name: name.to_owned(),
            count: coll
                .estimated_document_count()
                .run()
                .expect("Could not get estimated document count"),
            size: bytes_to_string(size),
            storage_size: bytes_to_string(storage_size),
        }
    }
}

pub(crate) fn create_connection(opts: CreateConnectionArgs) -> Result<(), Box<dyn Error>> {
    let mut conns = if let Some(conns) = read_connections() {
        conns
    } else {
        HashMap::new()
    };

    let mut new_conn = Connection::from_opts(opts);

    // error if this connection name is already in use
    if let Some(_) = conns.get(&new_conn.name) {
        return Err(ConnectionError::ConnectionExistsError.into());
    }

    // if there aren't any connections then this new one has to be the default
    if conns.is_empty() {
        eprintln!("This is the only connection, setting it to be the default");
        new_conn.default = true;
    }

    // if the new connection is set to default then we need to make sure all the other connections
    // are not default
    if new_conn.default {
        for (_name, conn) in conns.iter_mut() {
            conn.default = false;
        }
    }

    // update the connections hashmap and write it back to the filesystem
    conns.insert(new_conn.name.clone(), new_conn);
    write_connections(conns);

    Ok(())
}

fn bytes_to_string(size: i32) -> String {
    let units = ["B", "KB", "MB", "GB", "TB", "PB"];
    for (i, unit) in units.iter().enumerate() {
        if size / (1024_i32.pow(i as u32 + 1)) == 0 {
            let sizef = size as f64 / (1024_i32.pow(i as u32) as f64);
            return format!("{:.2}{}", sizef, unit);
        }
    }

    // more than 1024 PB, just use PB
    let sizef = size as f64 / (1024_i32.pow(5) as f64);
    format!("{:.2}{}", sizef, units[5])
}

pub(crate) fn estimate_document_count(args: &Cli, client: Client) -> mongodb::error::Result<()> {
    let Commands::EstimateDocumentCount {
        db: dbname,
        coll: collname,
    } = &args.command
    else {
        panic!("Expected a Commands::EstimateDocumentCount variant");
    };

    let db = client.database(dbname.as_str());
    let coll: Collection<Document> = db.collection(collname.as_str());
    let value = coll
        .estimated_document_count()
        .run()
        .expect("Could not get estimated document count for {dbname}.{collname}");
    println!("{value}");
    Ok(())
}

pub(crate) fn example(args: &Cli, client: Client) -> mongodb::error::Result<()> {
    let Commands::Example {
        db: dbname,
        coll: collname,
    } = &args.command
    else {
        panic!("Expected a Commands::Example variant");
    };

    let db = client.database(dbname.as_str());
    let coll: Collection<Document> = db.collection(collname.as_str());
    if let Some(example) = coll.find_one(doc! {}).run()? {
        for (key, value) in example.iter() {
            println!("  {key}: {value}");
        }
    } else {
        println!("No documents in {dbname}.{collname}");
    }
    Ok(())
}

pub(crate) fn example_filtered(args: &Cli, client: Client) -> mongodb::error::Result<()> {
    let Commands::ExampleFiltered {
        db: dbname,
        coll: collname,
        filter,
    } = &args.command
    else {
        panic!("Expected a Commands::ExampleFiltered variant");
    };

    let db = client.database(dbname.as_str());
    let coll: Collection<Document> = db.collection(collname.as_str());
    let doc =
        string_to_bson_doc(filter).expect("Could not convert given string to a bson Document");
    if let Some(example) = coll.find_one(doc).run()? {
        for (key, value) in example.iter() {
            println!("  {key}: {value}");
        }
    } else {
        println!("No documents in {dbname}.{collname} matched the filter");
    }
    Ok(())
}

pub(crate) fn list_collection_details(args: &Cli, client: Client) -> mongodb::error::Result<()> {
    let Commands::ListCollectionDetails { db: dbname } = &args.command else {
        panic!("Expected a Commands::ListCollectionDetails variant");
    };
    let db = client.database(dbname);
    let names = db.list_collection_names().run()?;
    let rows: Vec<DetailRow> = names.iter().map(|r| DetailRow::new(&db, r)).collect();
    let table_config = Settings::default()
        .with(Style::modern_rounded())
        .with(Alignment::right());
    let mut table = Table::new(rows);
    table.with(table_config);
    table.modify(Rows::first(), Alignment::center());
    println!("{}", table);
    Ok(())
}

pub(crate) fn list_collections(args: &Cli, client: Client) -> mongodb::error::Result<()> {
    let Commands::ListCollections { db: dbname } = &args.command else {
        panic!("Expected a Commands::ListCollections variant");
    };
    let db = client.database(dbname);
    let names = db.list_collection_names().run()?;
    if !names.is_empty() {
        for (i, name) in names.iter().enumerate() {
            println!("  {}) {}", i, name);
        }
    } else {
        println!("No collections in {dbname}");
    }
    Ok(())
}

pub(crate) fn list_connections() -> Result<(), Box<dyn Error>> {
    if let Some(conns) = read_connections() {
        let rows: Vec<&Connection> = conns.iter().map(|(_i, conn)| conn).collect();
        let table_config = Settings::default()
            .with(Style::modern_rounded())
            .with(Alignment::right());
        let mut table = Table::new(rows);
        table.with(table_config);
        table.modify(Rows::first(), Alignment::center());
        println!("{}", table);
    } else {
        println!("No connections found");
    }
    Ok(())
}

pub(crate) fn list_databases(_args: &Cli, client: Client) -> mongodb::error::Result<()> {
    let db_names = client.list_database_names().run()?;
    for (i, db) in db_names.iter().enumerate() {
        println!("  {}) {}", i, db);
    }
    Ok(())
}

pub(crate) fn string_to_bson_doc(s: &String) -> Result<Document, Box<dyn std::error::Error>> {
    let json_value: Value = serde_json::from_str(s.as_str())
        .map_err(|e| format!("Failed to parse JSON string: {}; Filter string: {}", e, s))?;
    let bson_document: Document = to_document(&json_value)
        .map_err(|e| format!("Failed to convert JSON Value to BSON Document: {}", e))?;
    Ok(bson_document)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_to_string() {
        assert_eq!(bytes_to_string(966), String::from("966.00B"));
        assert_eq!(bytes_to_string(1567), String::from("1.53KB"));
        assert_eq!(bytes_to_string(1567893), String::from("1.50MB"));
    }
}
