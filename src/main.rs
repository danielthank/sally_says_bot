mod handler;
mod utils;

use reqwest::StatusCode;
use std::{
    collections::HashMap,
    convert::Infallible,
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use teloxide::{
    dispatching::update_listeners,
    prelude::*,
    types::{Sticker, Update},
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::Filter;

type Db = Arc<Mutex<HashMap<String, Sticker>>>;

#[tokio::main]
async fn main() {
    run().await;
}

async fn handle_rejection(error: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    log::error!("Cannot process the request due to: {:?}", error);
    Ok(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn webhook<'a>(bot: AutoSend<Bot>) -> impl update_listeners::UpdateListener<Infallible> {
    // Heroku defines auto defines a port value
    let teloxide_token = env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN env variable missing");
    let port: u16 = env::var("PORT")
        .expect("PORT env variable missing")
        .parse()
        .expect("PORT value to be integer");
    // Heroku host example .: "heroku-ping-pong-bot.herokuapp.com"
    let host = env::var("HOST").expect("have HOST env variable");
    let path = format!("bot{}", teloxide_token);
    let url = format!("https://{}/{}", host, path);

    bot.set_webhook(url).await.expect("Cannot setup a webhook");

    let (tx, rx) = mpsc::unbounded_channel();

    let server = warp::post()
        .and(warp::path(path))
        .and(warp::body::json())
        .map(move |json: serde_json::Value| {
            if let Ok(update) = Update::try_parse(&json) {
                tx.send(Ok(update))
                    .expect("Cannot send an incoming update from the webhook")
            }
            StatusCode::OK
        })
        .recover(handle_rejection);

    let serve = warp::serve(server);

    let address = format!("0.0.0.0:{}", port);
    tokio::spawn(serve.run(address.parse::<SocketAddr>().unwrap()));
    UnboundedReceiverStream::new(rx)
}

async fn run() {
    pretty_env_logger::init();
    log::info!("Starting sally_says_bot...");

    let db_message: Db = Arc::new(Mutex::new(HashMap::new()));
    let db_callback_query = db_message.clone();
    let bot = Bot::from_env().auto_send();
    let listener = webhook(bot.clone()).await;

    Dispatcher::new(bot)
        .messages_handler(|rx| {
            UnboundedReceiverStream::new(rx).for_each_concurrent(None, move |message| {
                let db_message = db_message.clone();
                async move {
                    handler::messages_handler(message, db_message)
                        .await
                        .log_on_error()
                        .await;
                }
            })
        })
        .callback_queries_handler(|rx| {
            UnboundedReceiverStream::new(rx).for_each_concurrent(None, move |callback_query| {
                let db_callback_query = db_callback_query.clone();
                async move {
                    let message =
                        match handler::callback_queries_handler(&callback_query, db_callback_query)
                            .await
                        {
                            Ok(_) => "成功",
                            Err(_) => "失敗",
                        };
                    callback_query
                        .requester
                        .answer_callback_query(callback_query.update.id)
                        .text(message)
                        .show_alert(false)
                        .await
                        .log_on_error()
                        .await;
                }
            })
        })
        .dispatch_with_listener(
            listener,
            LoggingErrorHandler::with_custom_text("An error from the update listener"),
        )
        .await;
}
