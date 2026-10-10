fn main() {
    if let Err(error) =
        ios_native_build_support::run_cli(&std::env::args().skip(1).collect::<Vec<_>>())
    {
        println!(
            "{}",
            serde_json::json!({"schema_version":1,"status":"error","code":"build_failed","message":error})
        );
        std::process::exit(1);
    }
}
