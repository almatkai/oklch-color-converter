use oklch_beta_contract_harness::{direct, CoreContext, CorePaths};
use serde_json::json;
#[tokio::main]
async fn main() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    let ctx = CoreContext::new(
        CorePaths {
            data_dir: p.into(),
            config_dir: p.join("config"),
            resource_dir: p.join("resources"),
            temp_dir: p.join("tmp"),
            home_dir: p.into(),
        },
        None,
    );
    let mut state = json!({});
    direct(&ctx, "extension:run-command", &json!({}), &mut state)
        .await
        .unwrap();
    let view = direct(
        &ctx,
        "extension:search-text-changed",
        &json!({"searchText":"red"}),
        &mut state,
    )
    .await
    .unwrap();
    println!("{}", view);
}
