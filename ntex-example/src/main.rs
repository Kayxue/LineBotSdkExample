use dotenv::dotenv;
use line_bot_sdk_rust::{
    client::LINE,
    line_messaging_api::{
        apis::MessagingApiApi,
        models::{Message, ReplyMessageRequest, TextMessage},
    },
    line_webhook::models::{CallbackRequest, Event, MessageContent},
    parser::signature::validate_signature,
    support::ntex::Signature,
};
use ntex::{
    main,
    util::Bytes,
    web::{
        App, HttpServer, Responder, WebResponseError,
        error::{ErrorBadRequest, ErrorInternalServerError},
        post,
    },
};
use std::env;

#[post("/callback")]
async fn callback(
    signature: Signature,
    bytes: Bytes,
) -> Result<impl Responder, impl WebResponseError> {
    if let Err(_) = env::var("ACCESSTOKEN") {
        return Err(ErrorInternalServerError(
            "Can't get access token for Line Client",
        ));
    }
    if let Err(_) = env::var("CHANNELSECRET") {
        return Err(ErrorInternalServerError(
            "Can't get channel secret for Line Client",
        ));
    }

    let secret = &env::var("CHANNELSECRET").unwrap();

    let client = LINE::new(env::var("ACCESSTOKEN").unwrap());

    let body: &str = &String::from_utf8(bytes.to_vec()).unwrap();

    if !validate_signature(secret, &signature.key, &body) {
        return Err(ErrorBadRequest("Invalid signature"));
    }
    let request: Result<CallbackRequest, serde_json::Error> = serde_json::from_slice(&bytes);
    if let Ok(request) = request {
        for event in request.events {
            if let Event::MessageEvent(messageEvent) = event {
                if let MessageContent::TextMessageContent(text_message) = *messageEvent.message {
                    let reply_message_request = ReplyMessageRequest {
                        reply_token: messageEvent.reply_token.unwrap(),
                        messages: vec![Message::TextMessage(TextMessage {
                            text: text_message.text,
                            ..Default::default()
                        })],
                        notification_disabled: Some(false),
                    };
                    let _result = client
                        .messaging_api_client
                        .reply_message(reply_message_request)
                        .await;
                    // match result {
                    //     Ok(r) => println!("{:#?}", r),
                    //     Err(e) => println!("{:#?}", e),
                    // }
                };
            }
        }
    } else {
        return Err(ErrorBadRequest("Invalid request"));
    }
    Ok("Finished")
}

#[main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    HttpServer::new(|| App::new().service(callback))
        .bind(("0.0.0.0", 3000))?
        .run()
        .await
}
