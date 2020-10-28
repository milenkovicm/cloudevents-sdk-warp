use super::headers;
use bytes::Bytes;
use http::{header::HeaderName, HeaderMap};

use cloudevents::event::SpecVersion;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, Encoding, Error, MessageAttributeValue,
    MessageDeserializer, Result, StructuredDeserializer, StructuredSerializer,
};

use cloudevents::{message, Event};
use std::convert::TryFrom;

//pub(crate) type Request = http::Request<hyper::Body>;
/// Wrapper for [`HttpRequest`] that implements [`MessageDeserializer`] trait.
pub struct CERequestDeserializer<'a> {
    req: &'a HeaderMap,
    body: Bytes,
}

impl CERequestDeserializer<'_> {
    pub fn new(req: &HeaderMap, body: Bytes) -> CERequestDeserializer {
        CERequestDeserializer { req, body }
    }
}

impl<'a> BinaryDeserializer for CERequestDeserializer<'a> {
    fn deserialize_binary<R: Sized, V: BinarySerializer<R>>(self, mut visitor: V) -> Result<R> {
        if self.encoding() != Encoding::BINARY {
            return Err(message::Error::WrongEncoding {});
        }

        let spec_version = SpecVersion::try_from(
            unwrap_optional_header!(self.req, headers::SPEC_VERSION_HEADER).unwrap()?,
        )?;

        visitor = visitor.set_spec_version(spec_version.clone())?;

        let attributes = spec_version.attribute_names();

        for (hn, hv) in self
            .req
            .iter()
            .filter(|(hn, _)| headers::SPEC_VERSION_HEADER.ne(hn) && hn.as_str().starts_with("ce-"))
        {
            let name = &hn.as_str()["ce-".len()..];

            if attributes.contains(&name) {
                visitor = visitor.set_attribute(
                    name,
                    MessageAttributeValue::String(String::from(header_value_to_str!(hv)?)),
                )?
            } else {
                visitor = visitor.set_extension(
                    name,
                    MessageAttributeValue::String(String::from(header_value_to_str!(hv)?)),
                )?
            }
        }

        if let Some(hv) = self.req.get("content-type") {
            visitor = visitor.set_attribute(
                "datacontenttype",
                MessageAttributeValue::String(String::from(header_value_to_str!(hv)?)),
            )?
        }

        if self.body.len() != 0 {
            visitor.end_with_data(self.body.to_vec())
        } else {
            visitor.end()
        }
    }
}

impl<'a> StructuredDeserializer for CERequestDeserializer<'a> {
    fn deserialize_structured<R: Sized, V: StructuredSerializer<R>>(self, visitor: V) -> Result<R> {
        if self.encoding() != Encoding::STRUCTURED {
            return Err(message::Error::WrongEncoding {});
        }
        visitor.set_structured_event(self.body.to_vec())
    }
}

impl<'a> MessageDeserializer for CERequestDeserializer<'a> {
    fn encoding(&self) -> Encoding {
        if self
            .req
            .get("content-type")
            .map(|v| v.to_str().unwrap_or(""))
            .unwrap_or("")
            == "application/cloudevents+json"
        {
            Encoding::STRUCTURED
        } else if self
            .req
            .get::<&'static HeaderName>(&super::headers::SPEC_VERSION_HEADER)
            .is_some()
        {
            Encoding::BINARY
        } else {
            Encoding::UNKNOWN
        }
    }
}

/// Method to transform an incoming [`HttpRequest`] to [`Event`].
pub fn request_to_event(req: &HeaderMap, bytes: bytes::Bytes) -> std::result::Result<Event, Error> {
    //hyper::body::aggregate(&mut req.body()).await.unwrap();
    //hyper::body::to_bytes(req.body()).await.unwrap();

    //let mut bytes = BytesMut::new();
    // while let Some(item) = payload.next().await {
    //     bytes.extend_from_slice(&item?);
    // }
    MessageDeserializer::into_event(CERequestDeserializer::new(req, bytes))
}

// Extention Trait for [`HttpRequest`] which acts as a wrapper for the function [`request_to_event()`].
// #[async_trait(?Send)]
// pub trait RequestExt {
//     async fn to_event(
//         &self,
//         mut payload: web::Payload,
//     ) -> std::result::Result<Event, actix_web::error::Error>;
// }

// #[async_trait(?Send)]
// impl RequestExt for Request {
//     async fn to_event(
//         &self,
//         payload: web::Payload,
//     ) -> std::result::Result<Event, actix_web::error::Error> {
//         request_to_event(self, payload).await
//     }
// }
