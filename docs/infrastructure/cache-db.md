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
