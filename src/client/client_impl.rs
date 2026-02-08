use crate::adapter::{AdapterDispatcher, AdapterKind, ServiceType};
use crate::chat::{ChatOptions, ChatOptionsSet, ChatRequest, ChatResponse, ChatStreamResponse};
use crate::resolver::AuthData;
use crate::{Client, Error, ModelIden, Result, ServiceTarget};

/// Public AI Functions
impl Client {
	/// Returns all the model names for a given adapter kind.
	///
	/// IMPORTANT:
	/// - Besides the Ollama adapter, this will only look at a hardcoded static list of names for now.
	/// - For Ollama, it will currently make a live request to the default host/port (http://localhost:11434/v1/).
	/// - This function will eventually change to either take an endpoint or have another function to allow a custom endpoint.
	///
	/// Notes:
	/// - Since genai only supports Chat for now, the adapter implementation should attempt to remove the non-chat models.
	/// - Later, as genai adds more capabilities, we will have a `model_names(adapter_kind, Option<&[Skill]>)`
	///   that will take a list of skills like (`ChatText`, `ChatImage`, `ChatFunction`, `TextToSpeech`, ...).
	pub async fn all_model_names(&self, adapter_kind: AdapterKind) -> Result<Vec<String>> {
		let models = AdapterDispatcher::all_model_names(adapter_kind).await?;
		Ok(models)
	}

	/// Return the default model for a model_name str.
	/// This is used before
	pub fn default_model(&self, model_name: &str) -> Result<ModelIden> {
		// -- First get the default ModelInfo
		let adapter_kind = AdapterKind::from_model(model_name)?;
		let model_iden = ModelIden::new(adapter_kind, model_name);
		Ok(model_iden)
	}

	#[deprecated(note = "use `client.resolve_service_target(model_name)`")]
	pub async fn resolve_model_iden(&self, model_name: &str) -> Result<ModelIden> {
		let model = self.default_model(model_name)?;
		let target = self.config().resolve_service_target(model).await?;
		Ok(target.model)
	}

	pub async fn resolve_service_target(&self, model_name: &str) -> Result<ServiceTarget> {
		let model = self.default_model(model_name)?;
		self.config().resolve_service_target(model).await
	}

	/// Executes a chat.
	pub async fn exec_chat(
		&self,
		model: &str,
		chat_req: ChatRequest,
		// options not implemented yet
		options: Option<&ChatOptions>,
	) -> Result<ChatResponse> {
		let options_set = ChatOptionsSet::default()
			.with_chat_options(options)
			.with_client_options(self.config().chat_options());

		let model = self.default_model(model)?;
		let target = self.config().resolve_service_target(model).await?;
		let model = target.model.clone();

		let mut web_request_data =
			AdapterDispatcher::to_web_request_data(target.clone(), ServiceType::Chat, chat_req, options_set.clone())?;

		let auth_data = target.auth;
		if let AuthData::RequestOverride {
			url: override_url,
			headers: override_headers,
		} = auth_data
		{
			web_request_data.url = override_url;
			web_request_data.headers = override_headers;
		}

		// Add custom headers from options_set (merged after eventual RequestOverride)
		if let Some(custom_headers) = options_set.headers() {
			web_request_data.headers.extend(custom_headers);
		}

		let web_res = self
			.web_client()
			.do_post(&web_request_data.url, &web_request_data.headers, web_request_data.payload)
			.await
			.map_err(|webc_error| match webc_error {
				crate::webc::Error::ResponseFailedRateLimit { .. } => Error::WebModelRateLimit {
					model_iden: model.clone(),
					webc_error,
				},
				_ => Error::WebModelCall {
					model_iden: model.clone(),
					webc_error,
				},
			})?;

		let chat_res = AdapterDispatcher::to_chat_response(model, web_res, options_set)?;

		Ok(chat_res)
	}

	/// Executes a chat stream response.
	pub async fn exec_chat_stream(
		&self,
		model: &str,
		chat_req: ChatRequest, // options not implemented yet
		options: Option<&ChatOptions>,
	) -> Result<ChatStreamResponse> {
		let options_set = ChatOptionsSet::default()
			.with_chat_options(options)
			.with_client_options(self.config().chat_options());

		let model = self.default_model(model)?;
		let target = self.config().resolve_service_target(model).await?;
		let model = target.model.clone();
		let auth_data = target.auth.clone();

		let mut web_request_data =
			AdapterDispatcher::to_web_request_data(target, ServiceType::ChatStream, chat_req, options_set.clone())?;

		if let AuthData::RequestOverride {
			url: override_url,
			headers: override_headers,
		} = auth_data
		{
			web_request_data.url = override_url;
			web_request_data.headers = override_headers;
		};

		// Add custom headers from options_set
		if let Some(custom_headers) = options_set.headers() {
			web_request_data.headers.extend(custom_headers);
		}

		let reqwest_builder = self
			.web_client()
			.new_req_builder(&web_request_data.url, &web_request_data.headers, web_request_data.payload)
			.map_err(|webc_error| Error::WebModelCall {
				model_iden: model.clone(),
				webc_error,
			})?;

		let res = AdapterDispatcher::to_chat_stream(model.clone(), reqwest_builder, options_set).map_err(|e| {
			if let Error::ReqwestEventSource(ref es_err) = e {
				if let reqwest_eventsource::Error::InvalidStatusCode(status, _) = **es_err {
					if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
						return Error::WebModelRateLimit {
							model_iden: model.clone(),
							// Note: Here we don't have the body/headers for webc::Error easily from RequestEventSource error
							webc_error: crate::webc::Error::ResponseFailedRateLimit {
								body: String::new(),
								headers: Box::default(),
							},
						};
					}
				}
			}
			e
		})?;

		Ok(res)
	}
}
