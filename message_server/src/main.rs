use lambda_http::{
    handler,
    lambda_runtime::{self, Context},
    Body, IntoResponse, Request, RequestExt, Response,
};
use serde::Deserialize;
use serde_json::Value;

mod messages;

#[derive(Deserialize, Default)]
struct WriteMessageContent {
    user_id: String,
    message: String
}

#[derive(Deserialize, Default)]
struct GetMessageContent {
    user_id: String,
}

#[derive(Deserialize, Default)]
struct LoginContent {
    username: String
}

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(handler(write_message)).await?;
    Ok(())
}

async fn write_message(request: Request, _: Context) -> Result<Response<Body>, Error> {
    let payload_string = match request.body() {
        Body::Text(string) => string,
        _ => panic!("Request body is not text!")
    };
    let payload: Value = serde_json::from_str(payload_string)?;
    let command: &str = payload["command"].as_str().expect("'command' is missing or not a string!");
    let content = payload["content"].clone();
    let response: String = match command {
        "login" => {
            let content: LoginContent = serde_json::from_value(content).unwrap();
            let user_id = messages::get_or_create_user(&content.username).await;
            format!("{}", user_id)
        }
        "getMessage" => {
            let content: GetMessageContent = serde_json::from_value(content).unwrap();
            let message = messages::get_message(&content.user_id).await;
            message
        }
        "writeMessage" => {
            let content: WriteMessageContent = serde_json::from_value(content).unwrap();
            messages::write_message(&content.user_id, &content.message).await;
            "Written!".to_string()
        },
        _ => String::from("No 'command' provided in the message payload!"),
    };
    Ok(Response::new(
        format!("{}", response).into(),
    ))
}
