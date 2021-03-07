#![allow(clippy::style)]

pub mod config;

#[macro_use]
extern crate diesel;
extern crate dropshot;
extern crate serde_json;

/** Test module for privatemail package */
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

// LambdaRequest: Represents the incoming Request from AWS Lambda
//                This is deserialized into a struct payload
//
#[derive(Serialize, Debug)]
pub struct LambdaRequest<Data: DeserializeOwned> {
    #[serde(deserialize_with = "deserialize")]
    body: Data,
}

impl<Data: DeserializeOwned> LambdaRequest<Data> {
    pub fn body(&self) -> &Data {
        &self.body
    }
}

//
// LambdaResponse: The Outgoing response being passed by the Lambda
//
#[derive(Serialize, Debug)]
pub struct LambdaResponse {
    #[serde(rename = "isBase64Encoded")]
    is_base_64_encoded: bool,

    #[serde(rename = "statusCode")]
    status_code: u32,

    headers: HashMap<String, String>,
    body: String,
}

impl LambdaResponse {
    pub fn new() -> Self {
        LambdaResponse {
            is_base_64_encoded: false,
            status_code: 200,
            headers: HashMap::new(),
            body: "".to_owned(),
        }
    }

    pub fn with_status(mut self, code: u32) -> Self {
        self.status_code = code;
        self
    }

    pub fn with_header<S: Into<String>>(mut self, name: S, value: S) -> Self {
        self.headers.insert(name.into().to_ascii_lowercase(), value.into());
        self
    }

    pub fn with_json<D: Serialize>(mut self, data: D) -> Self {
        self.headers
            .entry("content-type".to_owned())
            .or_insert_with(|| "application/json".to_owned());

        self.body = serde_json::to_string(&data).unwrap();
        self
    }

    pub fn build(self) -> LambdaResponse {
        LambdaReponse {
            is_base_64_encoded: false,
            status_code: self.status_code,
            headers: self.headers,
            body: self.body,
        }
    }
}
