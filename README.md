# `mgfy`: A command line tool for quick inspection of MongoDB

`mgfy` (short for magnify) is built to be a quick and easy way to get
information from MongoDB without leaving the terminal. Have you ever forgotten
the precise name of the collection you need to write to, wanted to know how much
disk space is being taken used, or just needed an example document to remember
the schema that you're working with? That's exactly what `mgfy` does.

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

