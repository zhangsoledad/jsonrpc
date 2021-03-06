//! jsonrpc response
use serde::de::{Deserialize, Deserializer, Error as DeError};
use serde::ser::{Serialize, Serializer};
use serde_json::value::from_value;
use super::{Id, Value, Error, ErrorCode, Version};

/// Successful response
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Success {
	/// Protocol version
    #[serde(skip_serializing_if = "Option::is_none")]
	pub jsonrpc: Option<Version>,
	/// Result
	pub result: Value,
	/// Correlation id
	pub id: Id
}

/// Unsuccessful response
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Failure {
	/// Protocol Version
    #[serde(skip_serializing_if = "Option::is_none")]
	pub jsonrpc: Option<Version>,
	/// Error
	pub error: Error,
	/// Correlation id
	pub id: Id
}

/// Represents output - failure or success
#[derive(Debug, PartialEq, Clone)]
pub enum Output {
	/// Success
	Success(Success),
	/// Failure
	Failure(Failure),
}

impl Output {
	/// Creates new output given `Result`, `Id` and `Version`.
	pub fn from(result: Result<Value, Error>, id: Id, jsonrpc: Option<Version>) -> Self {
		match result {
			Ok(result) => Output::Success(Success {
				id: id,
				jsonrpc: jsonrpc,
				result: result,
			}),
			Err(error) => Output::Failure(Failure {
				id: id,
				jsonrpc: jsonrpc,
				error: error,
			}),
		}
	}

	/// Creates new failure output indicating malformed request.
	pub fn invalid_request(id: Id, jsonrpc: Option<Version>) -> Self {
		Output::Failure(Failure {
			id: id,
			jsonrpc: jsonrpc,
			error: Error::new(ErrorCode::InvalidRequest),
		})
	}
}

impl<'a> Deserialize<'a> for Output {
	fn deserialize<D>(deserializer: D) -> Result<Output, D::Error>
	where D: Deserializer<'a> {
		let v: Value = try!(Deserialize::deserialize(deserializer));
		from_value(v.clone()).map(Output::Failure)
			.or_else(|_| from_value(v).map(Output::Success))
			.map_err(|_| D::Error::custom("")) // types must match
	}
}

impl Serialize for Output {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		match *self {
			Output::Success(ref s) => s.serialize(serializer),
			Output::Failure(ref f) => f.serialize(serializer)
		}
	}
}

/// Synchronous response
#[derive(Debug, PartialEq)]
pub enum Response {
	/// Single response
	Single(Output),
	/// Response to batch request (batch of responses)
	Batch(Vec<Output>)
}

impl<'a> Deserialize<'a> for Response {
	fn deserialize<D>(deserializer: D) -> Result<Response, D::Error>
	where D: Deserializer<'a> {
		let v: Value = try!(Deserialize::deserialize(deserializer));
		from_value(v.clone()).map(Response::Batch)
			.or_else(|_| from_value(v).map(Response::Single))
			.map_err(|_| D::Error::custom("")) // types must match
	}
}

impl Serialize for Response {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		match *self {
			Response::Single(ref o) => o.serialize(serializer),
			Response::Batch(ref b) => b.serialize(serializer)
		}
	}
}

impl Response {
	/// Creates new `Response` with given error and `Version`
	pub fn from(error: Error, jsonrpc: Option<Version>) -> Self {
		Failure {
			id: Id::Null,
			jsonrpc: jsonrpc,
			error: error,
		}.into()
	}
}

impl From<Failure> for Response {
	fn from(failure: Failure) -> Self {
		Response::Single(Output::Failure(failure))
	}
}

impl From<Success> for Response {
	fn from(success: Success) -> Self {
		Response::Single(Output::Success(success))
	}
}

#[test]
fn success_output_serialize() {
	use serde_json;
	use serde_json::Value;

	let so = Output::Success(Success {
		jsonrpc: Some(Version::V2),
		result: Value::from(1),
		id: Id::Num(1)
	});

	let serialized = serde_json::to_string(&so).unwrap();
	assert_eq!(serialized, r#"{"jsonrpc":"2.0","result":1,"id":1}"#);
}

#[test]
fn success_output_deserialize() {
	use serde_json;
	use serde_json::Value;

	let dso = r#"{"jsonrpc":"2.0","result":1,"id":1}"#;

	let deserialized: Output = serde_json::from_str(dso).unwrap();
	assert_eq!(deserialized, Output::Success(Success {
		jsonrpc: Some(Version::V2),
		result: Value::from(1),
		id: Id::Num(1)
	}));
}

#[test]
fn failure_output_serialize() {
	use serde_json;

	let fo = Output::Failure(Failure {
		jsonrpc: Some(Version::V2),
		error: Error::parse_error(),
		id: Id::Num(1)
	});

	let serialized = serde_json::to_string(&fo).unwrap();
	assert_eq!(serialized, r#"{"jsonrpc":"2.0","error":{"code":-32700,"message":"Parse error"},"id":1}"#);
}

#[test]
fn failure_output_serialize_jsonrpc_1() {
	use serde_json;

	let fo = Output::Failure(Failure {
		jsonrpc: None,
		error: Error::parse_error(),
		id: Id::Num(1)
	});

	let serialized = serde_json::to_string(&fo).unwrap();
	assert_eq!(serialized, r#"{"error":{"code":-32700,"message":"Parse error"},"id":1}"#);
}

#[test]
fn failure_output_deserialize() {
	use serde_json;

	let dfo = r#"{"jsonrpc":"2.0","error":{"code":-32700,"message":"Parse error"},"id":1}"#;

	let deserialized: Output = serde_json::from_str(dfo).unwrap();
	assert_eq!(deserialized, Output::Failure(Failure {
		jsonrpc: Some(Version::V2),
		error: Error::parse_error(),
		id: Id::Num(1)
	}));
}

#[test]
fn single_response_deserialize() {
	use serde_json;
	use serde_json::Value;

	let dsr = r#"{"jsonrpc":"2.0","result":1,"id":1}"#;

	let deserialized: Response = serde_json::from_str(dsr).unwrap();
	assert_eq!(deserialized, Response::Single(Output::Success(Success {
		jsonrpc: Some(Version::V2),
		result: Value::from(1),
		id: Id::Num(1)
	})));
}

#[test]
fn batch_response_deserialize() {
	use serde_json;
	use serde_json::Value;

	let dbr = r#"[{"jsonrpc":"2.0","result":1,"id":1},{"jsonrpc":"2.0","error":{"code":-32700,"message":"Parse error"},"id":1}]"#;

	let deserialized: Response = serde_json::from_str(dbr).unwrap();
	assert_eq!(deserialized, Response::Batch(vec![
		Output::Success(Success {
			jsonrpc: Some(Version::V2),
			result: Value::from(1),
			id: Id::Num(1)
		}),
		Output::Failure(Failure {
			jsonrpc: Some(Version::V2),
			error: Error::parse_error(),
			id: Id::Num(1)
		})
	]));
}
