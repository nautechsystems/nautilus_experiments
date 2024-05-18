import pyarrow.parquet as pq
import sys


def extract_row_groups(source_file, target_file, num_groups):
    parquet_file = pq.ParquetFile(source_file)

    # Ensure we don't try to read more row groups than exist
    num_groups = min(num_groups, parquet_file.num_row_groups)

    # Read the first num_groups row groups
    row_groups = [parquet_file.read_row_group(i) for i in range(num_groups)]

    # Write the row groups to a new Parquet file
    with pq.ParquetWriter(target_file, row_groups[0].schema) as writer:
        for group in row_groups:
            writer.write_table(group)


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python extract_ts_init.py <parquet_file> <csv_file>")
        sys.exit(1)

    input_file = sys.argv[1]
    rows = int(sys.argv[2])

    extract_row_groups(input_file, f"{rows}-groups.parquet", rows)
