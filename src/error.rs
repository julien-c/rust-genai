use crate::adapter::AdapterKind;
use crate::chat::ChatRole;
use crate::{ModelIden, resolver, webc};
use derive_more::{Display, From};
use value_ext::JsonValueExtError;

/// GenAI main Result type alias (with genai::Error)
pub type Result<T> = core::result::Result<T, Error>;

/// Main GenAI error
#[derive(Debug, From, Display)]
#[allow(missing_docs)]
pub enum Error {
	// -- Chat Input
	#[display("Chat Request has no messages. (for model {model_iden}")]
	ChatReqHasNoMessages { model_iden: ModelIden },

	#[display("Last chat request message is not of Role 'user' (Actual role '{actual_role}') for model '{model_iden}'")]
	LastChatMessageIsNotUser {
		model_iden: ModelIden,
		actual_role: ChatRole,
	},

	#[display("Role '{role}' not supported for model '{model_iden}'")]
	MessageRoleNotSupported { model_iden: ModelIden, role: ChatRole },

	#[display("Content type not supported for model '{model_iden}'.\nCause: {cause}")]
	MessageContentTypeNotSupported { model_iden: ModelIden, cause: &'static str },

	#[display("JSON mode requested but no instruction/prompt provided.")]
	JsonModeWithoutInstruction,

	#[display("Failed to parse reasoning. Actual: '{actual}'")]
	ReasoningParsingError { actual: String },

	// -- Chat Output
	#[display("No chat response from model '{model_iden}'")]
	NoChatResponse { model_iden: ModelIden },

	#[display("Invalid JSON response element: {info}")]
	InvalidJsonResponseElement { info: &'static str },

	// -- Rate Limiting
	#[display("Rate limit exceeded for model '{model_iden}'. Please retry after some time.")]
	RateLimit { model_iden: ModelIden },

	// -- Auth
	#[display("Model '{model_iden}' requires an API key.")]
	RequiresApiKey { model_iden: ModelIden },

	#[display("No authentication resolver found for model '{model_iden}'.")]
	NoAuthResolver { model_iden: ModelIden },

	#[display("No authentication data available for model '{model_iden}'.")]
	NoAuthData { model_iden: ModelIden },

	// -- ModelMapper
	#[display("Model mapping failed for '{model_iden}'.\nCause: {cause}")]
	ModelMapperFailed {
		model_iden: ModelIden,
		cause: resolver::Error,
	},

	// -- Web Call error
	#[display("Web call failed for adapter '{adapter_kind}'.\nCause: {webc_error}")]
	WebAdapterCall {
		adapter_kind: AdapterKind,
		webc_error: webc::Error,
	},

	#[display("Web call failed for model '{model_iden}'.\nCause: {webc_error}")]
	WebModelCall {
		model_iden: ModelIden,
		webc_error: webc::Error,
	},

	// -- Chat Stream
	#[display("Failed to parse stream data for model '{model_iden}'.\nCause: {serde_error}")]
	StreamParse {
		model_iden: ModelIden,
		serde_error: serde_json::Error,
	},

	#[display("Error event in stream for model '{model_iden}'. Body: {body}")]
	StreamEventError {
		model_iden: ModelIden,
		body: serde_json::Value,
	},

	#[display("Web stream error for model '{model_iden}'.\nCause: {cause}")]
	WebStream { model_iden: ModelIden, cause: String },

	// -- Modules
	#[display("Resolver error for model '{model_iden}'.\nCause: {resolver_error}")]
	Resolver {
		model_iden: ModelIden,
		resolver_error: resolver::Error,
	},

	// -- Externals
	#[display("Failed to clone EventSource request: {_0}")]
	#[from]
	EventSourceClone(reqwest_eventsource::CannotCloneRequestError),

	#[display("JSON value extension error: {_0}")]
	#[from]
	JsonValueExt(JsonValueExtError),

	#[display("Reqwest EventSource error: {_0}")]
	ReqwestEventSource(Box<reqwest_eventsource::Error>),

	#[display("Serde JSON error: {_0}")]
	#[from]
	SerdeJson(serde_json::Error),
}

// region:    --- Error Boilerplate

// The Display trait is now derived via derive_more::Display
// impl core::fmt::Display for Error {
// 	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
// 		write!(fmt, "{self:?}")
// 	}
// }

impl std::error::Error for Error {}

impl Error {
	/// Check if a webc::Error represents a rate limit error and convert it appropriately
	pub fn from_webc_error_for_model(model_iden: ModelIden, webc_error: webc::Error) -> Self {
		if Self::is_rate_limit_error(&webc_error) {
			Error::RateLimit { model_iden }
		} else {
			Error::WebModelCall { model_iden, webc_error }
		}
	}

	/// Check if a webc::Error represents a rate limit error for adapter calls
	pub fn from_webc_error_for_adapter(adapter_kind: AdapterKind, webc_error: webc::Error) -> Self {
		// For adapter calls, we don't have a model_iden, so we stick with the existing behavior
		Error::WebAdapterCall {
			adapter_kind,
			webc_error,
		}
	}

	/// Detect if a webc::Error represents a rate limit condition
	fn is_rate_limit_error(webc_error: &webc::Error) -> bool {
		match webc_error {
			webc::Error::ResponseFailedStatus { status, body, .. } => {
				// Check for HTTP 429 (Too Many Requests)
				if status.as_u16() == 429 {
					return true;
				}

				// Check for common rate limit error messages in the response body
				let body_str = body.to_lowercase();
				body_str.contains("rate limit")
					|| body_str.contains("rate_limit")
					|| body_str.contains("too many requests")
					|| (body_str.contains("quota") && body_str.contains("exceeded"))
					|| (body_str.contains("rate") && body_str.contains("exceeded"))
					|| body_str.contains("throttle")
					|| body_str.contains("requests per")
					|| (body_str.contains("limit") && body_str.contains("exceeded"))
					|| body_str.contains("sending requests too quickly")
					|| body_str.contains("requests too quickly")
			}
			_ => false,
		}
	}
}

// endregion: --- Error Boilerplate
