use genai::adapter::AdapterKind;
use genai::webc;
use genai::{Error, ModelIden};
use reqwest::{StatusCode, header::HeaderMap};

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn test_rate_limit_error_detection_429_status() -> Result<()> {
	let model_iden = ModelIden::new(AdapterKind::Anthropic, "test-model");

	// Test HTTP 429 status code
	let webc_error = webc::Error::ResponseFailedStatus {
		status: StatusCode::TOO_MANY_REQUESTS,
		body: "Request limit exceeded".to_string(),
		headers: Box::new(HeaderMap::new()),
	};

	let error = Error::from_webc_error_for_model(model_iden.clone(), webc_error);

	match error {
		Error::RateLimit {
			model_iden: returned_model,
		} => {
			assert_eq!(returned_model.to_string(), model_iden.to_string());
		}
		_ => panic!("Expected RateLimit error, got: {:?}", error),
	}

	Ok(())
}

#[test]
fn test_rate_limit_error_detection_rate_limit_message() -> Result<()> {
	let model_iden = ModelIden::new(AdapterKind::Anthropic, "test-model");

	// Test various rate limit error messages
	let test_cases = vec![
		"Rate limit exceeded. Please try again later.",
		"rate_limit_error: Too many requests in 1 hour",
		"You have exceeded your quota for this month",
		"API rate limit exceeded",
		"Request throttled due to rate limiting",
		"You are sending requests too quickly",
		"Rate limit exceeded for requests per minute",
		"Limit exceeded: too many requests",
	];

	for message in test_cases {
		let webc_error = webc::Error::ResponseFailedStatus {
			status: StatusCode::BAD_REQUEST, // Even with non-429 status, should detect rate limit
			body: message.to_string(),
			headers: Box::new(HeaderMap::new()),
		};

		let error = Error::from_webc_error_for_model(model_iden.clone(), webc_error);

		match error {
			Error::RateLimit {
				model_iden: returned_model,
			} => {
				assert_eq!(returned_model.to_string(), model_iden.to_string());
			}
			_ => panic!("Expected RateLimit error for message '{}', got: {:?}", message, error),
		}
	}

	Ok(())
}

#[test]
fn test_rate_limit_error_detection_case_insensitive() -> Result<()> {
	let model_iden = ModelIden::new(AdapterKind::Anthropic, "test-model");

	// Test case insensitive detection
	let test_cases = vec![
		"RATE LIMIT EXCEEDED",
		"Rate_Limit_Error",
		"TOO MANY REQUESTS",
		"Quota EXCEEDED",
		"THROTTLE activated",
	];

	for message in test_cases {
		let webc_error = webc::Error::ResponseFailedStatus {
			status: StatusCode::BAD_REQUEST,
			body: message.to_string(),
			headers: Box::new(HeaderMap::new()),
		};

		let error = Error::from_webc_error_for_model(model_iden.clone(), webc_error);

		match error {
			Error::RateLimit {
				model_iden: returned_model,
			} => {
				assert_eq!(returned_model.to_string(), model_iden.to_string());
			}
			_ => panic!("Expected RateLimit error for message '{}', got: {:?}", message, error),
		}
	}

	Ok(())
}

#[test]
fn test_non_rate_limit_error_passthrough() -> Result<()> {
	let model_iden = ModelIden::new(AdapterKind::Anthropic, "test-model");

	// Test that non-rate-limit errors are passed through unchanged
	let webc_error = webc::Error::ResponseFailedStatus {
		status: StatusCode::INTERNAL_SERVER_ERROR,
		body: "Internal server error occurred".to_string(),
		headers: Box::new(HeaderMap::new()),
	};

	let error = Error::from_webc_error_for_model(model_iden.clone(), webc_error);

	match error {
		Error::WebModelCall {
			model_iden: returned_model,
			webc_error: _,
		} => {
			assert_eq!(returned_model.to_string(), model_iden.to_string());
		}
		_ => panic!("Expected WebModelCall error, got: {:?}", error),
	}

	Ok(())
}

#[test]
fn test_other_webc_error_types_passthrough() -> Result<()> {
	let model_iden = ModelIden::new(AdapterKind::Anthropic, "test-model");

	// Test that other types of webc errors are passed through unchanged
	let webc_error = webc::Error::ResponseFailedNotJson {
		content_type: "text/plain".to_string(),
	};

	let error = Error::from_webc_error_for_model(model_iden.clone(), webc_error);

	match error {
		Error::WebModelCall {
			model_iden: returned_model,
			webc_error: _,
		} => {
			assert_eq!(returned_model.to_string(), model_iden.to_string());
		}
		_ => panic!("Expected WebModelCall error, got: {:?}", error),
	}

	Ok(())
}

#[test]
fn test_adapter_error_passthrough() -> Result<()> {
	let adapter_kind = AdapterKind::Anthropic;

	// Test that adapter errors are passed through unchanged (no rate limit detection for adapter calls)
	let webc_error = webc::Error::ResponseFailedStatus {
		status: StatusCode::TOO_MANY_REQUESTS,
		body: "Rate limit exceeded".to_string(),
		headers: Box::new(HeaderMap::new()),
	};

	let error = Error::from_webc_error_for_adapter(adapter_kind, webc_error);

	match error {
		Error::WebAdapterCall {
			adapter_kind: returned_adapter,
			webc_error: _,
		} => {
			assert_eq!(returned_adapter, adapter_kind);
		}
		_ => panic!("Expected WebAdapterCall error, got: {:?}", error),
	}

	Ok(())
}
