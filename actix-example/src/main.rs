use actix_web::{
    App, Error, HttpResponse, HttpServer, error::ErrorBadRequest, middleware, post, web,
};
use dotenv::dotenv;
use line_bot_sdk_rust::{
    client::LINE,
    line_messaging_api::{
        apis::MessagingApiApi,
        models::{Message, ReplyMessageRequest, TextMessage},
    },
    line_webhook::models::{CallbackRequest, Event, MessageContent},
    parser::signature::validate_signature,
    support::actix::Signature,
};
use std::env;

#[post("/callback")]
async fn callback(signature: Signature, bytes: web::Bytes) -> Result<HttpResponse, Error> {
    // Get channel secret and access token by environment variable
    let channel_secret: &str =
        &env::var("CHANNELSECRET").expect("Failed to get LINE_CHANNEL_SECRET");
    let access_token: &str =
        &env::var("ACCESSTOKEN").expect("Failed to get LINE_CHANNEL_ACCESS_TOKEN");

    let line = LINE::new(access_token.to_string());

    let body: &str = &String::from_utf8(bytes.to_vec()).unwrap();

    if !validate_signature(channel_secret, &signature.key, body) {
        return Err(ErrorBadRequest("x-line-signature is invalid."));
    }

    let request: Result<CallbackRequest, serde_json::Error> = serde_json::from_slice(&bytes);
    match request {
        Err(err) => return Err(ErrorBadRequest(err.to_string())),
        Ok(req) => {
            println!("req: {req:#?}");
            for e in req.events {
                if let Event::MessageEvent(message_event) = e {
                    if let MessageContent::TextMessageContent(text_message) = *message_event.message
                    {
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
                            Ok(r) => println!("{:#?}", r),
                            Err(e) => println!("{:#?}", e),
                        }
                    };
                };
            }
        }
    }

    Ok(HttpResponse::Ok().body("ok"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(callback)
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}
