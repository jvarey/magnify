use crate::cli::CreateConnectionArgs;
use crate::errors::ConnectionError;
use anyhow::Result;
use directories::ProjectDirs;
use mongodb::sync::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufWriter, Write};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Connection {
    pub(crate) name: String,
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) protocol: String,
    pub(crate) default: bool,
}

impl Connection {
    pub(crate) fn from_name(name_opt: Option<&str>) -> Result<Self> {
        let conn = if let Some(conns) = read_connections() {
            if let Some(name) = name_opt {
                get_by_name(&conns, name)?.clone()
            } else {
                get_default_connection(&conns)?.clone()
            }
        } else {
            return Err(ConnectionError::NoConnections.into());
        };

        Ok(conn)
    }

    pub(crate) fn from_opts(opts: CreateConnectionArgs) -> Self {
        Self {
            name: opts.name,
            host: opts.host,
            port: opts.port,
            protocol: opts.protocol,
            default: opts.default,
        }
    }

    pub(crate) fn to_uri(&self) -> String {
        format!("{}://{}:{}", self.protocol, self.host, self.port)
    }

    pub(crate) fn connect(&self) -> mongodb::error::Result<Client> {
        Client::with_uri_str(self.to_uri())
    }
}

#[derive(Debug, Clone, Tabled)]
pub(crate) struct ConnectionDisplay {
    #[tabled(rename = "Default")]
    pub(crate) default: String,
    #[tabled(rename = "Name")]
    pub(crate) name: String,
    #[tabled(rename = "Host")]
    pub(crate) host: String,
    #[tabled(rename = "Port")]
    pub(crate) port: u16,
    #[tabled(rename = "Protocol")]
    pub(crate) protocol: String,
}

impl ConnectionDisplay {
    pub(crate) fn from_connection(conn: &Connection) -> Self {
        Self {
            default: Self::default_string(conn.default),
            name: conn.name.clone(),
            host: conn.host.clone(),
            port: conn.port,
            protocol: conn.protocol.clone(),
        }
    }

    fn default_string(default: bool) -> String {
        if default {
            "✅".to_string()
        } else {
            "".to_string()
        }
    }
}

fn get_by_name<'a>(conns: &'a HashMap<String, Connection>, name: &str) -> Result<&'a Connection> {
    conns
        .get(name)
        .ok_or_else(|| ConnectionError::ConnectionDoesNotExist.into())
}

fn get_default_connection(conns: &HashMap<String, Connection>) -> Result<&Connection> {
    for conn in conns.values() {
        if conn.default {
            return Ok(conn);
        }
    }
    Err(ConnectionError::NoDefaultConnection.into())
}

pub(crate) fn read_connections() -> Option<HashMap<String, Connection>> {
    let proj_dir =
        ProjectDirs::from("com", "jvarey", "mgfy").expect("Could not parse project directory");
    let fname = proj_dir.data_dir().join("connections.json");
    if fname.exists() {
        let content = fs::read_to_string(fname).expect("Could not read connections.json");
        return Some(serde_json::from_str(&content).expect("Could not parse JSON"));
    }
    None
}

pub(crate) fn write_connections(conns: &HashMap<String, Connection>) -> Result<()> {
    let Some(proj_dir) = ProjectDirs::from("com", "jvarey", "mgfy") else {
        return Err(ConnectionError::WriteConnection.into());
    };
    fs::create_dir_all(proj_dir.data_dir())?;
    let fname = proj_dir.data_dir().join("connections.json");

    let Ok(file) = fs::File::create(fname) else {
        return Err(ConnectionError::WriteConnection.into());
    };
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &conns)?;
    writer.flush()?;
    Ok(())
}
