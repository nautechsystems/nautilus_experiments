# Death by row groups

Use the python script to extract row group information from the parquet files using pyarrow.

```bash
pip install -r requirements.txt
python extract_ts_init.py 126-groups.parquet 126-groups-python.csv
python extract_ts_init.py 127-groups.parquet 127-groups-python.csv
```

Run the rust executable to extract row group information from the parquet files using datafusion.

```bash
cargo run 126-groups.parquet > 126-groups-rust.csv
cargo run 127-groups.parquet > 127-groups-rust.csv
```

Ideally there should be no difference between the csv files for the row groups. However, 126 works properly. But 127 gives different results for Python and Rust.

This shows that indeed there's no difference with 126 groups.

```bash
diff 126-groups-rust.csv 126-groups-python.csv # no diff
diff 126-groups-rust.csv 126-groups-python.csv # big diff, things crazy
```

We can also make sure that these are in fact from the same data source with just one extra row group with this command which shows 127 groups python has only one extra entry at the end.

```bash
diff 126-groups-python.csv 127-groups-python.csv
```
