use std::net::SocketAddr;

use axum::{Router, body::Bytes, http::StatusCode, response::IntoResponse, routing::post};
use bot_sdk_line::{
    client::LINE,
    messaging_api_line::{
        apis::MessagingApiApi,
        models::{Message, ReplyMessageRequest, TextMessage},
    },
    parser::signature::validate_signature,
    support::axum::Signature,
    webhook_line::models::{CallbackRequest, Event, MessageContent},
};
use dotenv::dotenv;
use std::env;
use tokio::{main, net::TcpListener};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

async fn callback(Signature(key): Signature, bytes: Bytes) -> impl IntoResponse {
    let channel_secret: &str =
        &env::var("CHANNELSECRET").expect("Failed to get LINE_CHANNEL_SECRET");
    let access_token: &str =
        &env::var("ACCESSTOKEN").expect("Failed to get LINE_CHANNEL_ACCESS_TOKEN");

    let line = LINE::new(access_token.to_string());

    let body: &str = &String::from_utf8(bytes.to_vec()).unwrap();

    if !validate_signature(channel_secret, &key, body) {
        return (StatusCode::BAD_REQUEST, "x-line-signature is invalid.");
    }

    let request: Result<CallbackRequest, serde_json::Error> = serde_json::from_slice(&bytes);
    if let Ok(req) = request {
        println!("req: {req:#?}");
        for e in req.events {
            if let Event::MessageEvent(message_event) = e {
                if let MessageContent::TextMessageContent(text_message) = *message_event.message {
                    let reply_message_request = ReplyMessageRequest {
                        reply_token: message_event.reply_token.unwrap(),
                        messages: vec![Message::TextMessage(TextMessage {
                            text: text_message.text,
                            ..Default::default()
                        })],
                        notification_disabled: Some(false),
                    };
                    let result = line
                        .messaging_api_client
                        .reply_message(reply_message_request)
                        .await;
                    match result {
                        Ok(r) => {
                            println!("{:#?}", r);
                        }
                        Err(e) => {
                            println!("{:#?}", e);
                        }
                    }
                }
            }
        }
        return (StatusCode::OK, "OK")
    }
    (StatusCode::BAD_REQUEST, "Body parsing failed")
}

#[main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new().route("/callback", post(callback));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {addr}");

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
