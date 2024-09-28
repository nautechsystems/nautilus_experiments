# Death by row groups

The parquet file is sorted on the `ts_init` column. We want to stream data from the file in ascending order of `ts_init`. However, we do not want to sort the data in-memory since it is already sorted. To achieve this we use the following datafusion configuration.

```rust
    let session_cfg =
        SessionConfig::new().set_str("datafusion.optimizer.repartition_file_scans", "false");
    let session_ctx = SessionContext::new_with_config(session_cfg);
    let parquet_options = ParquetReadOptions::<'_> {
        skip_metadata: Some(false),
        file_sort_order: vec![vec![datafusion_expr::SortExpr {
            expr: col("ts_init"),
            asc: true,
            nulls_first: false,
        }]],
        ..Default::default()
    };
```

This works well when there is no filter clause in the query. The below commands will pass.

```
cargo run 127-groups.parquet > 127-groups-rust.csv
python check_invariant.py 127-groups-rust.csv
```

However, when there is a filter clause in the query. The row groups are not read in-order causing the ascending order invariant to fail.

```
cargo run 127-groups.parquet filter > 127-groups-rust.csv
python check_invariant.py 127-groups-rust.csv #fail
```

Run the rust executable to extract row group information from the parquet files using datafusion.

# Helper utils

Make a smaller parquet file by extracting the first n row groups.

```bash
python extract_groups.py <parquet-file> <num-rows>
```

Check that the invariant holds over the row groups i.e. the row group timestamps are in ascending order and not overlapping.

```bash
python check_invariant.py <csv-file>
```
