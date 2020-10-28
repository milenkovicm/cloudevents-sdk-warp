#[macro_use]
mod headers;

mod server_request;
mod server_response;

pub mod reply {
    use crate::server_response::event_to_response;

    use cloudevents::Event;
    use http::StatusCode;
    use warp::reply::Response;

    ///
    /// # Serializes `CE` as a http response
    /// 
    /// ```
    /// use cloudevents_sdk_warp::{filter, reply};
    /// use warp::Filter;
    /// use warp::Reply;
    /// 
    /// let routes = warp::any()
    ///    .and(filter::ce_event())
    ///    .map(|event| reply::ce_event(event));
    /// ```
    
    pub fn ce_event(event: Event) -> Response {
        match event_to_response(event) {
            Ok(response) => response,
            Err(e) => {
                warp::http::response::Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(hyper::body::Body::from(e.to_string()))
                    .unwrap()
                //StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }

    #[cfg(test)]
    mod tests {

        use cloudevents::{EventBuilder, EventBuilderV10};
        use serde_json::json;
        use std::str::FromStr;
        use url::Url;

        #[test]
        fn test_response() {
            let input = EventBuilderV10::new()
                .id("0001")
                .ty("example.test")
                .source(Url::from_str("http://localhost/").unwrap())
                .extension("someint", "10")
                .build()
                .unwrap();

            let resp = super::ce_event(input);

            assert_eq!(
                resp.headers()
                    .get("ce-specversion")
                    .unwrap()
                    .to_str()
                    .unwrap(),
                "1.0"
            );
            assert_eq!(
                resp.headers().get("ce-id").unwrap().to_str().unwrap(),
                "0001"
            );
            assert_eq!(
                resp.headers().get("ce-type").unwrap().to_str().unwrap(),
                "example.test"
            );
            assert_eq!(
                resp.headers().get("ce-source").unwrap().to_str().unwrap(),
                "http://localhost/"
            );
            assert_eq!(
                resp.headers().get("ce-someint").unwrap().to_str().unwrap(),
                "10"
            );
        }

        #[tokio::test]
        async fn test_response_with_full_data() {
            let j = json!({"hello": "world"});

            let input = EventBuilderV10::new()
                .id("0001")
                .ty("example.test")
                .source(Url::from_str("http://localhost").unwrap())
                .data("application/json", j.clone())
                .extension("someint", "10")
                .build()
                .unwrap();

            let resp = super::ce_event(input);

            assert_eq!(
                resp.headers()
                    .get("ce-specversion")
                    .unwrap()
                    .to_str()
                    .unwrap(),
                "1.0"
            );
            assert_eq!(
                resp.headers().get("ce-id").unwrap().to_str().unwrap(),
                "0001"
            );
            assert_eq!(
                resp.headers().get("ce-type").unwrap().to_str().unwrap(),
                "example.test"
            );
            assert_eq!(
                resp.headers().get("ce-source").unwrap().to_str().unwrap(),
                "http://localhost/"
            );
            assert_eq!(
                resp.headers()
                    .get("content-type")
                    .unwrap()
                    .to_str()
                    .unwrap(),
                "application/json"
            );
            assert_eq!(
                resp.headers().get("ce-someint").unwrap().to_str().unwrap(),
                "10"
            );

            let (_, body) = resp.into_parts();
            let body = hyper::body::to_bytes(body).await.unwrap();

            assert_eq!(j.to_string().as_bytes(), body);
        }
    }
}

pub mod filter {
    use crate::server_request::request_to_event;

    use cloudevents::Event;
    use warp::http::HeaderMap;
    use warp::Filter;
    use warp::Rejection;

    #[derive(Debug)]
    pub struct CEFilterError {
        message: String,
    }

    impl warp::reject::Reject for CEFilterError {}


    ///
    /// # Extracts `CE` event from incoming request
    /// 
    /// ```
    /// use cloudevents_sdk_warp::filter;
    /// use warp::Filter;
    /// use warp::Reply;
    /// 
    /// let routes = warp::any()
    ///    .and(filter::ce_event())
    ///    .map(|event| {
    ///         // do something with the event
    ///     }
    ///     );
    /// ```
    /// 
    
    pub fn ce_event() -> impl Filter<Extract = (Event,), Error = Rejection> + Copy {
        warp::header::headers_cloned()
            .and(warp::body::bytes())
            .and_then(create_event)
    }

    async fn create_event(headers: HeaderMap, body: bytes::Bytes) -> Result<Event, Rejection> {
        match request_to_event(&headers, body) {
            Ok(event) => Ok(event),
            Err(e) => Err(warp::reject::custom(CEFilterError {
                message: e.to_string(),
            })),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::ce_event;
        use url::Url;
        use warp::test;

        use chrono::Utc;
        use cloudevents::{EventBuilder, EventBuilderV10};
        use serde_json::json;
        use std::str::FromStr;

        #[tokio::test]
        async fn test_request() {
            let time = Utc::now();
            let expected = EventBuilderV10::new()
                .id("0001")
                .ty("example.test")
                .source("http://localhost/")
                //TODO this is required now because the message deserializer implictly set default values
                // As soon as this defaulting doesn't happen anymore, we can remove it (Issues #40/#41)
                .time(time)
                .extension("someint", "10")
                .build()
                .unwrap();

            let result = test::request()
                .method("POST")
                .header("ce-specversion", "1.0")
                .header("ce-id", "0001")
                .header("ce-type", "example.test")
                .header("ce-source", "http://localhost/")
                .header("ce-someint", "10")
                .header("ce-time", time.to_rfc3339())
                .filter(&ce_event())
                .await
                .unwrap();

            assert_eq!(expected, result);
        }

        #[tokio::test]
        async fn test_request_with_full_data() {
            let time = Utc::now();
            let j = json!({"hello": "world"});

            let expected = EventBuilderV10::new()
                .id("0001")
                .ty("example.test")
                .source(Url::from_str("http://localhost").unwrap())
                //TODO this is required now because the message deserializer implictly set default values
                // As soon as this defaulting doesn't happen anymore, we can remove it (Issues #40/#41)
                .time(time)
                .data("application/json", j.to_string().into_bytes())
                .extension("someint", "10")
                .build()
                .unwrap();

            let result = test::request()
                .method("POST")
                .header("ce-specversion", "1.0")
                .header("ce-id", "0001")
                .header("ce-type", "example.test")
                .header("ce-source", "http://localhost")
                .header("ce-someint", "10")
                .header("ce-time", time.to_rfc3339())
                .header("content-type", "application/json")
                .json(&j)
                .filter(&ce_event())
                .await
                .unwrap();

            assert_eq!(expected, result);
        }
    }
}
