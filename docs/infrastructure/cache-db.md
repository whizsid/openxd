# Cache DB

We can use file system for caching images and resource files. But we can not store
the structural data of our oxd file as files. Because we have to query those data
fast and also we have to modify those data fast. Therefore a database approach is
well suitable for caching. But we have to take a decision on which DB is fast as
and embedded database and a cloud database.

## Embedded DB

- SQLite
- RocksDB
- Sled
- tikv
- Surreal

## Cloud DB

- MongoDB
- MySQL
- Surreal

## Decision

SQLite and RocksDB are very matured projects as embedded database engines. But RocksDB 
is just a key-value DB, we can not query our components using coordinates. Sled and tikv
are Rust embedded databases. But both
database engines are just key-value storages. Surreal DB is a new innovative DB engine
that built entirely using Rust. It provides a powerful query system which help us to query
components by using coordinates. Also it using RocksDB as a BE key-value storage and they
hoping to move to Sled. Also there is a cloud version of Surreal DB. So we can share the
same queries and same data structures between our cloud and desktop version applications.
