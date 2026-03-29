use crate::cli::CreateConnectionArgs;
use crate::connections::{read_connections, write_connections, Connection};
use crate::errors::ConnectionError;
use anyhow::{anyhow, Context, Result};
use mongodb::{
    bson::{self, doc, to_document, Document},
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
    fn try_new(db: &Database, name: &str) -> Result<Self> {
        let cmd = doc! { "collStats": name, "scale": 1 };
        let coll: Collection<Document> = db.collection(name);
        let stats = db.run_command(cmd).run()?;
        // ordinarily we would just do `stats.get_i32("storageSize")?`, but the result has a decent
        // chance of overflowing an i32. if it would overflow, mongo actually saves sizes as i64.
        // this way we always get an i64 out of mongo.
        let storage_size = stats
            .get("storageSize")
            .and_then(|v| match v {
                bson::Bson::Int32(n) => Some(i64::from(*n)),
                bson::Bson::Int64(n) => Some(*n),
                _ => None,
            })
            .ok_or(anyhow!("storageSize missing or not numeric"))?;
        let size = stats
            .get("size")
            .and_then(|v| match v {
                bson::Bson::Int32(n) => Some(i64::from(*n)),
                bson::Bson::Int64(n) => Some(*n),
                _ => None,
            })
            .ok_or(anyhow!("size missing or not numeric"))?;
        Ok(Self {
            name: name.to_owned(),
            count: coll.estimated_document_count().run()?,
            size: bytes_to_string(size),
            storage_size: bytes_to_string(storage_size),
        })
    }
}

pub(crate) fn collections(dbname: &str, verbose: bool, client: &Client) -> Result<()> {
    let db = client.database(dbname);
    let names = db.list_collection_names().run()?;
    if verbose {
        let mut rows = Vec::new();
        for name in &names {
            rows.push(DetailRow::try_new(&db, name)?);
        }
        let table_config = Settings::default()
            .with(Style::modern_rounded())
            .with(Alignment::right());
        let mut table = Table::new(rows);
        table.with(table_config);
        table.modify(Rows::first(), Alignment::center());
        println!("{table}");
    } else {
        for name in &names {
            println!("{name}");
        }
    }
    Ok(())
}

pub(crate) fn create_connection(opts: CreateConnectionArgs) -> Result<()> {
    let mut conns = read_connections().unwrap_or_default();
    let mut new_conn = Connection::from_opts(opts);

    if conns.contains_key(&new_conn.name) {
        return Err(ConnectionError::ConnectionExists.into());
    }

    // if there aren't any connections then this new one has to be the default
    if conns.is_empty() {
        eprintln!("This is the only connection, setting it to be the default");
        new_conn.default = true;
    }

    // if the new connection is set to default then we need to make sure all the other connections
    // are not default
    if new_conn.default {
        for conn in conns.values_mut() {
            conn.default = false;
        }
    }

    // update the connections hashmap and write it back to the filesystem
    conns.insert(new_conn.name.clone(), new_conn);
    write_connections(&conns)?;

    Ok(())
}

fn bytes_to_string(size: i64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB", "PB"];
    for (i, unit) in units.iter().enumerate() {
        if size / (1024_i64).pow(i as u32 + 1) == 0 {
            let sizef = size as f64 / (1024_i64.pow(i as u32) as f64);
            return format!("{sizef:.2}{unit}");
        }
    }

    // more than 1024 PB, just use PB
    let sizef = size as f64 / (1024_i64.pow(5) as f64);
    format!("{:.2}{}", sizef, units[5])
}

pub(crate) fn estimate_document_count(dbname: &str, collname: &str, client: &Client) -> Result<()> {
    let db = client.database(dbname);
    let coll: Collection<Document> = db.collection(collname);
    let value = coll.estimated_document_count().run()?;
    println!("{value}");
    Ok(())
}

pub(crate) fn example(dbname: &str, collname: &str, client: &Client) -> Result<()> {
    let db = client.database(dbname);
    let coll: Collection<Document> = db.collection(collname);
    if let Some(example) = coll.find_one(doc! {}).run()? {
        for (key, value) in &example {
            println!("  {key}: {value}");
        }
    } else {
        println!("No documents in {dbname}.{collname}");
    }
    Ok(())
}

pub(crate) fn example_filtered(
    dbname: &str,
    collname: &str,
    filter: &str,
    client: &Client,
) -> Result<()> {
    let db = client.database(dbname);
    let coll: Collection<Document> = db.collection(collname);
    let doc =
        string_to_bson_doc(filter).expect("Could not convert given string to a bson Document");
    if let Some(example) = coll.find_one(doc).run()? {
        for (key, value) in &example {
            println!("  {key}: {value}");
        }
    } else {
        println!("No documents in {dbname}.{collname} matched the filter");
    }
    Ok(())
}

pub(crate) fn list_connections() {
    if let Some(conns) = read_connections() {
        let rows = conns.values();
        let table_config = Settings::default()
            .with(Style::modern_rounded())
            .with(Alignment::right());
        let mut table = Table::new(rows);
        table.with(table_config);
        table.modify(Rows::first(), Alignment::center());
        println!("{table}");
    } else {
        println!("No connections found");
    }
}

pub(crate) fn list_databases(client: &Client) -> Result<()> {
    let db_names = client.list_database_names().run()?;
    for (i, db) in db_names.iter().enumerate() {
        println!("  {}) {}", i + 1, db);
    }
    Ok(())
}

pub(crate) fn string_to_bson_doc(s: &str) -> Result<Document> {
    let json_value: Value = serde_json::from_str(s)
        .with_context(|| format!("Failed to parse JSON string; Filter string: {s}"))?;
    let bson_document: Document = to_document(&json_value)
        .with_context(|| "Failed to convert JSON Value to BSON Document")?;
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
