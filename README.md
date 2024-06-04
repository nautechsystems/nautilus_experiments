## Query for the efficient query

`data.parquet` is sorted by timestamp on field `ts_init` with row groups of 5000 and size 100 MB. **The goal is to find a query, config combination that streams data in the sorted order in a memory efficient manner**. There are two parameters to consider -

* `order` - Adds an `ORDER BY ts_init` clause to the sql query
* `repartition` - Configures datafusion session to read row groups from the file in parallel

The binary reads the file and prints the number of rows from the first row group. We compare different configurations for their time and memory usage. You can run the following queries to get these values.

```
/usr/bin/time -v cargo run --release --bin single_row_group data.parquet order repartition
/usr/bin/time -v cargo run --release --bin single_row_group data.parquet order
/usr/bin/time -v cargo run --release --bin single_row_group data.parquet 
/usr/bin/time -v cargo run --release --bin single_row_group data.parquet repartition
```

These are the values on my system.

| order | repartition | wall time (s) | memory (mb) | read sorted order |
| -- | -- | -- | -- | -- |
| true | true | 0.84 | 654 | ✅ |
| true | false | 1.19 | 944 | ✅ |
| false | false | 0.21 | 45 | ✅ |
| false | true | 0.33 | 151 | ❌ |

The `full_file` binary reads the whole file and counts the total number of rows.

```
/usr/bin/time -v cargo run --release --bin full_file data.parquet order repartition
/usr/bin/time -v cargo run --release --bin full_file data.parquet order
/usr/bin/time -v cargo run --release --bin full_file data.parquet 
/usr/bin/time -v cargo run --release --bin full_file data.parquet repartition
```

| order | repartition | wall time (s) | memory (mb) | read sorted order |
| -- | -- | -- | -- | -- |
| true | true | 1.24 | 654 | ✅ |
| true | false | 1.65 | 944 | ✅ |
| false | false | 0.55 | 45 | ✅ |
| false | true | 0.41 | 151 | ❌ |
