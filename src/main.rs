use std::env::{self, args};

use datafusion::arrow::array::{Int64Array, UInt64Array};
use datafusion::arrow::{
    array::{Array, ArrayRef},
    datatypes::{DataType, Schema},
    error::ArrowError,
    ipc::writer::StreamWriter,
    record_batch::RecordBatch,
};
use datafusion::execution::context::SessionContext;
use datafusion::execution::options::ParquetReadOptions;
use datafusion::{
    error::Result, logical_expr::expr::Sort, physical_plan::SendableRecordBatchStream, prelude::*,
};
use futures::StreamExt;

pub fn extract_column<'a, T: Array + 'static>(
    cols: &'a [ArrayRef],
    column_key: &'static str,
    column_index: usize,
    expected_type: DataType,
) -> &'a T {
    let column_values = cols.get(column_index).unwrap();
    let downcasted_values = column_values.as_any().downcast_ref::<T>().unwrap();
    downcasted_values
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
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

    let file_path = args().nth(1).expect("Missing file path argument");
    session_ctx
        .register_parquet("data", &file_path, parquet_options)
        .await
        .unwrap();

    // Set filter option in query
    let filter_query = env::args().find(|v| v == "filter").is_some();
    let df = if filter_query {
        session_ctx.sql("SELECT * FROM data where ts_init >= 1701388832486000000 AND ts_init <= 1701392194001000000").await.unwrap()
    } else {
        session_ctx.sql("SELECT * FROM data").await.unwrap()
    };

    let mut stream = df.execute_stream().await.unwrap().enumerate();

    println!("index,start_ts,end_ts,group_size");
    while let Some((batch_count, Ok(batch))) = stream.next().await {
        let cols = batch.columns();
        let num_rows = batch.num_rows();
        let ts_init_values = extract_column::<UInt64Array>(cols, "ts_init", 5, DataType::UInt64);
        println!(
            "{},{},{},{}",
            batch_count,
            ts_init_values.value(0),
            ts_init_values.value(num_rows - 1),
            num_rows
        );
    }
}
