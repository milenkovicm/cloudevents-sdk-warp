# Cloud Event Support For Warp

This crate provides [cloud event](https://github.com/cloudevents/sdk-rust) support for [warp server](https://github.com/seanmonstar/warp).

This implementation is heavily inspired by [cloudevents-sdk-actix-web](https://github.com/cloudevents/sdk-rust/tree/master/cloudevents-sdk-actix-web)
implementation.

## Usage examples

Basically event can be extracted from request or returned as response

Extracting cloud event from a request and returning it back to sender:

```rust
use cloudevents_sdk_warp::{filter, reply};
use warp::Filter;

//  cargo run --example example_request
#[tokio::main]
async fn main() {
    let routes = warp::any()
        // extracting event from request
        .and(filter::to_event())
        // returning event back
        .map(|event| reply::from_event(event));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
```

executing `http` request:

```http
POST http://localhost:3030/
ce-specversion: 1.0
ce-id: 2
ce-type: example.event
ce-source: url://example_response/
content-type: application/json

{
  "age": 43,
  "name": "John Doe",
  "phones": [
    "+44 1234567",
    "+44 2345678"
  ]
}
```

should produce response similar to:

```
HTTP/1.1 200 OK
ce-specversion: 1.0
ce-id: 2
ce-type: example.event
ce-source: url://example_response/
content-type: application/json
content-length: 93
date: Wed, 28 Oct 2020 09:14:33 GMT

{
  "age": 43,
  "name": "John Doe",
  "phones": [
    "+44 1234567",
    "+44 2345678"
  ]
}
```

If we want to return created event back:

```rust
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
            Ok(event) => Ok(reply::from_event(event)),
            Err(e) => Ok(warp::reply::with_status(
                e.to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response()),
        }
    });

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
```

[examples](examples/) directory contains full example.
