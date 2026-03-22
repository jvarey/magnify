use crate::cli::{Cli, Commands};
use mongodb::{
    bson::{doc, Document},
    sync::{Client, Collection, Database},
};
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
    fn new(db: &Database, name: &String) -> Self {
        let cmd = doc! { "collStats": name.clone() };
        let coll: Collection<Document> = db.collection(name.as_str());
        let stats = db
            .run_command(cmd)
            .run()
            .expect("Could not get collection stats");
        let storage_size = stats
            .get_i32("storageSize")
            .expect("Could not get storage size");
        let size = stats.get_i32("size").expect("Could not get size");
        Self {
            name: name.clone(),
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
    for i in 0..=5 {
        if size / (1024_i32.pow(i as u32 + 1)) == 0 {
            let sizef = size as f64 / (1024_i32.pow(i as u32) as f64);
            return format!("{:.2}{}", sizef, units[i]);
        }
    }

    // more than 1024 PB, just use PB
    let sizef = size as f64 / (1024_i32.pow(5) as f64);
    format!("{:.2}{}", sizef, units[5])
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
    if names.len() > 0 {
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
