use log::info;

pub fn init() {
    std::env::set_var("RUST_LOG_STYLE", "timestamp=%Y-%m-%d %H:%M:%S");

    match env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init()
    {
        Ok(_) => {}
        Err(e) => {
            info!("env_logger init failed {}", e)
        }
    }
}
