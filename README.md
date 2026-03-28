# `mgfy`: A command line tool for quick inspection of MongoDB

`mgfy` (short for magnify) is built to be a quick and easy way to get
information from MongoDB without leaving the terminal. Have you ever forgotten
the precise name of the collection you need to write to, wanted to know how much
disk space is being taken used, or just needed an example document to remember
the schema that you're working with? That's exactly what `mgfy` does.

## Installation

Right now `mgfy` can only be installed using the [Rust
Toolchain](https://rust-lang.org/tools/install/). Like all other binary crates,
it can be installed with:

```shell
cargo install mgfy
```

## Connections

`mgfy` allows you to define and save parameters to easily switch between
separate connections. Each connection is defined by the following fields:

| Field    | Description |
|----------|-------------|
| name     | A unique name for the connection; this will be used to specify which
connection to use in subsequent commands |
| host     | hostname used in the connection string (e.g. "localhost") |
| port     | port used in the connection string (e.g. 27017, MongoDB's default |
| protocol | The first portion of the connection string (default: "mongodb")|
| default  | Flag for whether this should be the default connection |

There are a few commands for managing connections, they are:
  1) `create-connection`: uses the specified parameters in creating a new
  connection
  2) `list-connections`: lists the connections that have been saved
  3) TODO `remove-connection`: deletes a saved connection
  4) TODO `set-default-connection`: sets the named connection to be the default

## Usage

There are a number of commands that are built into `mgfy` so far:
  1) `estimate-document-count`: will return the document count from the
  collection metadata
  2) `example`: gets an example document from the collection without filtering
  3) `exmaple-filtered`: gets an example document from the collection after
  filtering on the user-provided query
  4) `list-collection-details`: returns the name, document count, size, and
  allocated storage for each collection in a database
  5) `list-collections`: returns the name of all the collections in a database
  6) `list-databases`: returns all of the database names

Here's the help text:
```shell
$ mgfy -h
Usage: magnify [OPTIONS] <COMMAND>

Commands:
  estimate-document-count  Estimate document count
  example                  Get an example document
  example-filtered         Get an example document after filtering
  list-collection-details  Get detailed information on each collection
  list-collections         List the collections in a database
  list-databases           List the databases
  help                     Print this message or the help of the given subcommand(s)

Options:
      --protocol <PROTOCOL>  [default: mongodb]
      --host <HOST>          [default: localhost]
      --port <PORT>          [default: 20667]
  -h, --help                 Print help
  -V, --version              Print version
```

## Filtering

Getting Rust to interpret an input string as JSON is not trivial. The following
is a working example:
```shell
mgfy example-filtered foo bar '{"Hello": {"$exists": true}}'
```

