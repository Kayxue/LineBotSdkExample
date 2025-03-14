use dotenv::dotenv;
use line_bot_sdk_rust::{
    client::LINE,
    line_messaging_api::{
        apis::MessagingApiApi,
        models::{Message, ReplyMessageRequest, TextMessage},
    },
    line_webhook::models::{CallbackRequest, Event, MessageContent, event},
    parser::signature::validate_signature,
    support::{
        XitcaError::{BadRequest, InternalServerError},
        xitca::Signature,
    },
};
use std::env;
use xitca_web::{App, bytes::Bytes, codegen::route, error::Error};

#[route("/callback",method = post)]
async fn callback(signature: Signature<'_>, bytes: Bytes) -> Result<&'static str, Error> {
    if let Err(_) = env::var("ACCESSTOKEN") {
        return Err(InternalServerError::new("Can't get access token for Line Client").into());
    }
    if let Err(_) = env::var("CHANNELSECRET") {
        return Err(InternalServerError::new("Can't get channel secret for Line Client").into());
    }

    let secret = &env::var("CHANNELSECRET").unwrap();

    let client = LINE::new(env::var("ACCESSTOKEN").unwrap());

    let body: &str = &String::from_utf8(bytes.to_vec()).unwrap();

    if !validate_signature(&secret, signature.key, body) {
        return Err(BadRequest::new("Invalid signature").into());
    }

    let request: Result<CallbackRequest, serde_json::Error> = serde_json::from_str(&body);
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
        return Err(BadRequest::new("Invalid request").into());
    }

    Ok("Finished")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    //Load env
    dotenv().ok();

    App::new()
        .at_typed(callback)
        .serve()
        .bind(("0.0.0.0", 3000))?
        .run()
        .await
}
