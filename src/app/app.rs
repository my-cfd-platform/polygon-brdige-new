use std::{collections::HashMap, sync::Arc};

use rust_extensions::AppStates;
use tokio::sync::Mutex;

use crate::{settings::SettingsModel, setup_and_start_ws, setup_price_tcp_server, TcpConnection};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");

pub struct AppContext {
    pub app_states: Arc<AppStates>,
    pub settings: Arc<SettingsModel>,
    pub connections: Mutex<HashMap<i32, Arc<TcpConnection>>>,
}

impl AppContext {
    pub fn new(settings: Arc<SettingsModel>) -> AppContext {
        AppContext {
            app_states: Arc::new(AppStates::create_initialized()),
            settings,
            connections: Mutex::new(HashMap::new()),
        }
    }
}

pub async fn setup_and_start(app: &Arc<AppContext>) {
    let app_for_spawn = app.clone();

    setup_and_start_ws(app_for_spawn.clone()).await;
    let tcp_server = setup_price_tcp_server(&app);
    tcp_server.start().await;

    app.app_states.set_initialized();
}
