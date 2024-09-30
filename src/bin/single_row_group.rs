use std::env;

use datafusion::execution::{context::SessionContext, options::ParquetReadOptions};
use datafusion::logical_expr::expr::Sort;
use datafusion::prelude::*;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    // Read file_path from command-line arguments
    let file_path = env::args().nth(1).expect("Missing file_path argument");
    // Set repartitioning option in session configuration
    let repartition_file_scan = if env::args().find(|v| v == "repartition").is_some() {
        "true"
    } else {
        "false"
    };
    // Set sql query ordering clause
    let sql_query = if env::args().find(|v| v == "order").is_some() {
        "SELECT * FROM data ORDER BY ts_init"
    } else {
        "SELECT * FROM data"
    };
    // Explain query
    let explain = if env::args().find(|v| v == "explain").is_some() {
        true
    } else {
        false
    };

    let session_cfg = SessionConfig::new()
        .set_str(
            "datafusion.optimizer.repartition_file_scans",
            repartition_file_scan,
        )
        .set_str("datafusion.optimizer.prefer_existing_sort", "true");
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

    session_ctx
        .register_parquet("data", &file_path, parquet_options)
        .await
        .unwrap();
    let query = session_ctx.sql(sql_query).await.unwrap();

    if explain {
        let explain = query.explain(false, false).unwrap();
        println!("{:#?}", explain);
    } else {
        let mut batch_stream = query.execute_stream().await.unwrap();
        let batch = batch_stream.next().await.unwrap().unwrap();
        println!("{}", batch.num_rows());
    }
}
