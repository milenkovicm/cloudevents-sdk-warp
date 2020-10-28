use super::headers;

use cloudevents::event::SpecVersion;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, Error, MessageAttributeValue, Result,
    StructuredSerializer,
};
use cloudevents::Event;

use warp::http::HeaderValue;
use warp::hyper::Body;
use warp::reply::Response;

use http::header::HeaderName;
use http::response::Builder;

use std::str::FromStr;

pub struct CEResponseSerializer {
    builder: Builder,
}

impl CEResponseSerializer {
    #[allow(dead_code)]
    pub fn from(builder: Builder) -> CEResponseSerializer {
        CEResponseSerializer { builder }
    }

    fn new() -> Self {
        CEResponseSerializer {
            builder: http::Response::builder(),
        }
    }
}

impl BinarySerializer<Response> for CEResponseSerializer {
    fn set_spec_version(mut self, spec_version: SpecVersion) -> Result<Self> {
        self.builder = self.builder.header(
            headers::SPEC_VERSION_HEADER.clone(),
            str_to_header_value!(spec_version.as_str())?,
        );
        Ok(self)
    }

    fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.builder = self.builder.header(
            headers::ATTRIBUTES_TO_HEADERS.get(name).unwrap().clone(),
            str_to_header_value!(value.to_string().as_str())?,
        );
        Ok(self)
    }

    fn set_extension(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.builder = self.builder.header(
            attribute_name_to_header!(name)?,
            str_to_header_value!(value.to_string().as_str())?,
        );
        Ok(self)
    }

    fn end_with_data(self, bytes: Vec<u8>) -> Result<Response> {
        self.builder
            .body(Body::from(bytes))
            .map_err(|e| cloudevents::message::Error::Other {
                source: Box::new(e),
            })
    }

    fn end(self) -> Result<Response> {
        self.builder
            .body(Body::empty())
            .map_err(|e| cloudevents::message::Error::Other {
                source: Box::new(e),
            })
    }
}

impl StructuredSerializer<Response> for CEResponseSerializer {
    fn set_structured_event(self, bytes: Vec<u8>) -> Result<Response> {
        Ok(self
            .builder
            .header(
                http::header::CONTENT_TYPE,
                headers::CLOUDEVENTS_JSON_HEADER.clone(),
            )
            .body(Body::from(bytes))
            .map_err(|e| cloudevents::message::Error::Other {
                source: Box::new(e),
            })?)
    }
}

pub fn event_to_response(event: Event) -> std::result::Result<Response, Error> {
    BinaryDeserializer::deserialize_binary(event, CEResponseSerializer::new())
}

