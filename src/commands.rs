use crate::cli::{Cli, Commands};
use mongodb::{
    bson::{doc, to_document, Document},
    sync::{Client, Collection, Database},
};
use serde_json::Value;
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

pub(crate) fn estimate_document_count(args: &Cli) -> mongodb::error::Result<()> {
    let Commands::EstimateDocumentCount {
        db: dbname,
        coll: collname,
    } = &args.command
    else {
        panic!("Expected a Commands::EstimateDocumentCount variant");
    };

    let client = connect(args)?;
    let db = client.database(dbname.as_str());
    let coll: Collection<Document> = db.collection(collname.as_str());
    let value = coll
        .estimated_document_count()
        .run()
        .expect("Could not get estimated document count for {dbname}.{collname}");
    println!("{value}");
    Ok(())
}

pub(crate) fn example(args: &Cli) -> mongodb::error::Result<()> {
    let Commands::Example {
        db: dbname,
        coll: collname,
    } = &args.command
    else {
        panic!("Expected a Commands::Example variant");
    };

    let client = connect(args)?;
    let db = client.database(dbname.as_str());
    let coll: Collection<Document> = db.collection(collname.as_str());
    if let Some(example) = coll.find_one(doc! {}).run()? {
        println!("Example document from {dbname}.{collname}");
        for (key, value) in example.iter() {
            println!("  {key}: {value}");
        }
    } else {
        println!("No documents in {dbname}.{collname}");
    }
    Ok(())
}

pub(crate) fn example_filtered(args: &Cli) -> mongodb::error::Result<()> {
    let Commands::ExampleFiltered {
        db: dbname,
        coll: collname,
        filter,
    } = &args.command
    else {
        panic!("Expected a Commands::ExampleFiltered variant");
    };

    let client = connect(args)?;
    let db = client.database(dbname.as_str());
    let coll: Collection<Document> = db.collection(collname.as_str());
    let doc =
        string_to_bson_doc(filter).expect("Could not convert given string to a bson Document");
    if let Some(example) = coll.find_one(doc).run()? {
        println!("Example document from {dbname}.{collname}");
        for (key, value) in example.iter() {
            println!("  {key}: {value}");
        }
    } else {
        println!("No documents in {dbname}.{collname} matched the filter");
    }
    Ok(())
}

pub(crate) fn list_collection_details(args: &Cli) -> mongodb::error::Result<()> {
    let Commands::ListCollectionDetails { db: dbname } = &args.command else {
        panic!("Expected a Commands::ListCollectionDetails variant");
    };
    let client = connect(args)?;
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

pub(crate) fn list_collections(args: &Cli) -> mongodb::error::Result<()> {
    let Commands::ListCollections { db: dbname } = &args.command else {
        panic!("Expected a Commands::ListCollections variant");
    };
    let client = connect(args)?;
    let db = client.database(dbname);
    let names = db.list_collection_names().run()?;
    if !names.is_empty() {
        println!("Collections in {dbname}");
        for (i, name) in names.iter().enumerate() {
            println!("  {}) {}", i, name);
        }
    } else {
        println!("No collections in {dbname}");
    }
    Ok(())
}

pub(crate) fn list_databases(args: &Cli) -> mongodb::error::Result<()> {
    let client = connect(args)?;
    let db_names = client.list_database_names().run()?;
    println!("Databases:");
    for (i, db) in db_names.iter().enumerate() {
        println!("  {}) {}", i, db);
    }
    Ok(())
}

pub(crate) fn connect(args: &Cli) -> mongodb::error::Result<Client> {
    let uri = format!("{}://{}:{}", args.protocol, args.host, args.port);
    let client = Client::with_uri_str(uri)?;
    Ok(client)
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
