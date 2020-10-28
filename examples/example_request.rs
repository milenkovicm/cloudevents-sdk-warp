#![deny(warnings)]
use cloudevents_sdk_warp::{filter, reply};
use warp::Filter;

//  cargo run --example example_request
#[tokio::main]
async fn main() {
    let routes = warp::any()
        // extracting event from request
        .and(filter::ce_event())
        // returning event back
        .map(|event| reply::event(event));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
