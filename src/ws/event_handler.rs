use std::sync::Arc;

use my_web_socket_client::WsConnection;
use polygon_io_client::ws::{
    PolygonEventHandler, PolygonWsClient, PolygonWsError, PolygonWsSettings, SendEventMessage,
    WsDataEvent,
};

use crate::{AppContext, BidAsk};

pub enum SocketType {
    Forex,
    Stocks,
}

pub struct MyEventHandler {
    pub app: Arc<AppContext>,
    pub socket_type: SocketType,
}

#[async_trait::async_trait]
impl PolygonEventHandler for MyEventHandler {
    async fn on_data(&self, event: WsDataEvent, _: &Arc<WsConnection>) {
        let bid_ask_message: Option<BidAsk> = match event {
            WsDataEvent::Status(_) => None,
            WsDataEvent::ForexQuoteTick(forex) => Some(forex.into()),
            WsDataEvent::StockQuoteTick(quote) => Some(quote.into()),
        };

        if let Some(mut bid_ask) = bid_ask_message {
            let internal_id = self.app.settings.instruments_mapping.get(&bid_ask.id);
            if let Some(internal_id) = internal_id {
                bid_ask.id = internal_id.clone();
                for connection in self.app.connections.lock().await.values() {
                    connection
                        .send(crate::BidAskContract::BidAsk(bid_ask.clone()))
                        .await;
                }
            }
        }
    }
    async fn on_connected(&self, connection: &Arc<WsConnection>) {
        println!("Handle on connected");
        let message = match self.socket_type {
            SocketType::Forex => SendEventMessage::ForexQuotesSubscribe(None).as_message().into(),
            SocketType::Stocks => SendEventMessage::StockQuotesSubscribe(None).as_message().into(),
        };

        println!("Send message: {:?}", message);
        connection.send_message(message).await;
    }
    async fn on_disconnected(&self, _: &Arc<WsConnection>) {
        println!("Handle disconedted");
    }
    async fn on_error(&self, error: PolygonWsError, _: &Arc<WsConnection>) {
        println!("Handle error. Error: {:?}", error);
    }
}

pub async fn setup_and_start_ws(app: Arc<AppContext>) {
    let forex_settings = PolygonWsSettings {
        url: format!("{}/forex", app.settings.ws_settings_base_url),
        token_key: app.settings.polygon_token.clone(),
    };

    let stock_settings = PolygonWsSettings {
        url: format!("{}/stocks", app.settings.ws_settings_base_url),
        token_key: app.settings.polygon_token.clone(),
    };

    let forex_socket = PolygonWsClient::new(
        Arc::new(forex_settings),
        Arc::new(MyEventHandler {
            app: app.clone(),
            socket_type: SocketType::Forex,
        }),
    );

    let stock_socket = PolygonWsClient::new(
        Arc::new(stock_settings),
        Arc::new(MyEventHandler {
            app,
            socket_type: SocketType::Stocks,
        }),
    );

    PolygonWsClient::start(Arc::new(forex_socket));
    PolygonWsClient::start(Arc::new(stock_socket));

}
