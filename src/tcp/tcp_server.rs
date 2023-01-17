use std::{net::SocketAddr, sync::Arc};

use my_tcp_sockets::{tcp_connection::SocketConnection, TcpServer};
use rust_extensions::AppStates;

use crate::{AppContext, BidAskContract, SourceFeedSerializer};

use super::Callback;

pub type TcpConnection = SocketConnection<BidAskContract, SourceFeedSerializer>;

pub struct PriceRouterTcpServer {
    pub tcp_server: TcpServer<BidAskContract, SourceFeedSerializer>,
    pub app: Arc<AppContext>,
}

impl PriceRouterTcpServer {
    pub fn new(
        tcp_server: TcpServer<BidAskContract, SourceFeedSerializer>,
        app: Arc<AppContext>,
    ) -> Self {
        Self { tcp_server, app }
    }

    pub async fn start(&self) {
        let app_states = Arc::new(AppStates::create_initialized());

        self.tcp_server
            .start(
                Arc::new(SourceFeedSerializer::new),
                Arc::new(Callback::new(self.app.clone())),
                app_states,
                my_logger::LOGGER.clone(),
            )
            .await;

        println!("TCP server started");
    }
}

pub fn setup_price_tcp_server(app: &Arc<AppContext>) -> PriceRouterTcpServer {
    let tcp_server: TcpServer<BidAskContract, SourceFeedSerializer> = TcpServer::new(
        "PolygonBrdige".to_string(),
        SocketAddr::from(([0, 0, 0, 0], 8085)),
    );

    return PriceRouterTcpServer {
        tcp_server,
        app: app.clone(),
    };
}
