#![deny(warnings)]
use cloudevents::{EventBuilder, EventBuilderV10};
use cloudevents_sdk_warp::reply;
use http::StatusCode;
use serde_json::json;
use warp::Filter;
use warp::Reply;

//  cargo run --example example_request
#[tokio::main]
async fn main() {
    let routes = warp::any().map(|| {
        let event = EventBuilderV10::new()
            .id("1")
            .source(url::Url::parse("url://example_response/").unwrap())
            .ty("example.ce")
            .data(
                mime::APPLICATION_JSON.to_string(),
                json!({
                    "name": "John Doe",
                    "age": 43,
                    "phones": [
                        "+44 1234567",
                        "+44 2345678"
                    ]
                }),
            )
            .build();

        match event {
            Ok(event) => Ok(reply::ce_event(event)),
            Err(e) => Ok(warp::reply::with_status(
                e.to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response()),
        }
    });

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
