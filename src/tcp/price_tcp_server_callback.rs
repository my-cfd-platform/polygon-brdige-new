use std::sync::Arc;

use my_tcp_sockets::{ConnectionEvent, SocketEventCallback};

use crate::{AppContext, BidAskContract, SourceFeedSerializer};

pub struct Callback {
    pub app: Arc<AppContext>,
}

impl Callback {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl SocketEventCallback<BidAskContract, SourceFeedSerializer> for Callback {
    async fn handle(
        &self,
        connection_event: ConnectionEvent<BidAskContract, SourceFeedSerializer>,
    ) {
        match connection_event {
            ConnectionEvent::Connected(connection) => {
                let mut write_access = self.app.connections.lock().await;
                println!("New connection {}", connection.id);
                write_access.insert(connection.id, connection);
            }
            ConnectionEvent::Disconnected(connection) => {
                let mut write_access = self.app.connections.lock().await;
                write_access.remove(&connection.id);
                println!("Disconnected {}", connection.id);
            }
            ConnectionEvent::Payload {
                connection,
                payload,
            } => {
                if payload.is_ping() {
                    connection.send(BidAskContract::Pong).await;
                }
                println!("Received payload from {:?}", payload);
            }
        }
    }
}
