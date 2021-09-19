# magg

Perform queries/updates on MongoDB collections through the command-line!

## Demo

### Basic usage

1. Create a config file.
   You can also pass the values as arguments.

```shell
hbina@akarin:~/git/magg$ echo '{
>   "connection_uri": "mongodb://localhost:27017",
>   "database_name": "demo-database-name",
>   "collection_name": "demo-collection-name",
>   "pipelines": []
> }
> ' > demo-config.json
```

2. Generate dummy data to be inserted.
   Support UNIX pipes!

```shell
hbina@akarin:~/git/magg$ echo '{ "{{word}}" : "{{date}}"}' | npx datamaker --iterations 10 | cargo run -- --config-file ./demo-config.json create
    Finished dev [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/magg --config-file ./demo-config.json create`
Successfully inserted 10 documents with _id:
"61474a8a18e446c5dcd760e7"
"61474a8a18e446c5dcd760ec"
"61474a8a18e446c5dcd760e5"
"61474a8a18e446c5dcd760eb"
"61474a8a18e446c5dcd760e9"
"61474a8a18e446c5dcd760ed"
"61474a8a18e446c5dcd760e8"
"61474a8a18e446c5dcd760ea"
"61474a8a18e446c5dcd760e4"
"61474a8a18e446c5dcd760e6"
```

3. Perform queries.
   Support MongoDB pipelines.

```shell
hbina@akarin:~/git/magg$ cargo run -- --config-file ./demo-config.json aggregate --pipeline '[]'
    Finished dev [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/magg --config-file ./demo-config.json aggregate --pipeline '[]'`
{ "_id": "61474a8a18e446c5dcd760e4", "loving": "1990-09-26" }
{ "_id": "61474a8a18e446c5dcd760e5", "icon": "2005-09-03" }
{ "_id": "61474a8a18e446c5dcd760e6", "for": "1997-08-19" }
{ "_id": "61474a8a18e446c5dcd760e7", "generators": "1993-09-29" }
{ "_id": "61474a8a18e446c5dcd760e8", "double": "2018-12-20" }
{ "_id": "61474a8a18e446c5dcd760e9", "choosing": "2020-08-16" }
{ "_id": "61474a8a18e446c5dcd760ea", "maintained": "1999-07-22" }
{ "_id": "61474a8a18e446c5dcd760eb", "merchant": "1999-05-31" }
{ "_id": "61474a8a18e446c5dcd760ec", "musicians": "1979-01-22" }
{ "_id": "61474a8a18e446c5dcd760ed", "reel": "2005-05-01" }
```

### Save common pipelines in a configuration file

Save a configuration to save time.
Must adhere to this structure.

```shell
hbina@akarin:~/git/magg$ cat config.json.example
{
  "connection_uri": "mongodb://localhost:27017",
  "database_name": "database-name",
  "collection_name": "collection-name",
  "pipelines": [
    {
      "name": "get everything",
      "description": "gets everything from the collection",
      "stages": []
    },
    {
      "name": "get everything",
      "description": "gets everything from the collection",
      "stages": [
        {
          "$project": {
            "_id": 1
          }
        }
      ]
    }
  ]
}
```

## Help

```shell
hbina@akarin:~/git/magg$ cargo run -- help
    Finished dev [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/magg help`
magg 0.1.0



USAGE:
    magg [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --collection-name <collection-name>
        --config-file <config-file>
        --connection-uri <connection-uri>
        --database-name <database-name>

SUBCOMMANDS:
    aggregate    Perform aggregation on a collection
    create       Insert documents into the collection
    help         Prints this message or the help of the given subcommand(s)
```
