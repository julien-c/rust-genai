use genai::{Error, ModelIden};
use genai::adapter::AdapterKind;
use genai::webc;
use reqwest::{StatusCode, header::HeaderMap};

fn test_anthropic_rate_limit_scenarios() {
    println!("Testing Anthropic rate limit scenarios...\n");
    
    let model_iden = ModelIden::new(AdapterKind::Anthropic, "claude-3-5-haiku-latest");
    
    // Test real Anthropic error messages
    let anthropic_error_messages = vec![
        (429, "Rate limit exceeded. Please retry your request after a brief wait."),
        (429, "You have exceeded your hourly limit."),
        (400, "You exceeded your current quota, please check your plan and billing details."),
        (429, "{'type': 'error', 'error': {'type': 'rate_limit_error', 'message': 'Number of requests per minute exceeded'}}"),
        (400, "API rate limit exceeded"),
    ];
    
    for (status, message) in anthropic_error_messages {
        let webc_error = webc::Error::ResponseFailedStatus {
            status: StatusCode::from_u16(status).unwrap(),
            body: message.to_string(),
            headers: Box::new(HeaderMap::new()),
        };
        
        let error = Error::from_webc_error_for_model(model_iden.clone(), webc_error);
        
        match error {
            Error::RateLimit { model_iden } => {
                println!("✓ DETECTED rate limit for {}: {}", model_iden, message);
            }
            _ => {
                println!("✗ MISSED rate limit detection for: {}", message);
            }
        }
    }
}

fn test_openai_rate_limit_scenarios() {
    println!("\nTesting OpenAI rate limit scenarios...\n");
    
    let model_iden = ModelIden::new(AdapterKind::OpenAI, "gpt-4");
    
    // Test real OpenAI error messages
    let openai_error_messages = vec![
        (429, "Rate limit reached for requests"),
        (429, "Too many requests per minute"),
        (400, "You exceeded your current quota"),
        (429, "Rate limit exceeded"),
        (400, "Billing quota exceeded"),
    ];
    
    for (status, message) in openai_error_messages {
        let webc_error = webc::Error::ResponseFailedStatus {
            status: StatusCode::from_u16(status).unwrap(),
            body: message.to_string(),
            headers: Box::new(HeaderMap::new()),
        };
        
        let error = Error::from_webc_error_for_model(model_iden.clone(), webc_error);
        
        match error {
            Error::RateLimit { model_iden } => {
                println!("✓ DETECTED rate limit for {}: {}", model_iden, message);
            }
            _ => {
                println!("✗ MISSED rate limit detection for: {}", message);
            }
        }
    }
}

fn test_non_rate_limit_errors() {
    println!("\nTesting non-rate-limit errors (should NOT be detected as rate limits)...\n");
    
    let model_iden = ModelIden::new(AdapterKind::OpenAI, "gpt-4");
    
    // Test non-rate-limit error messages
    let non_rate_limit_errors = vec![
        (400, "Invalid request format"),
        (401, "Authentication failed"),
        (404, "Model not found"),
        (500, "Internal server error"),
        (503, "Service unavailable"),
    ];
    
    for (status, message) in non_rate_limit_errors {
        let webc_error = webc::Error::ResponseFailedStatus {
            status: StatusCode::from_u16(status).unwrap(),
            body: message.to_string(),
            headers: Box::new(HeaderMap::new()),
        };
        
        let error = Error::from_webc_error_for_model(model_iden.clone(), webc_error);
        
        match error {
            Error::RateLimit { .. } => {
                println!("✗ INCORRECTLY detected rate limit for: {}", message);
            }
            _ => {
                println!("✓ CORRECTLY ignored non-rate-limit error: {}", message);
            }
        }
    }
}

fn main() {
    println!("=== Rate Limit Error Detection Test ===\n");
    
    test_anthropic_rate_limit_scenarios();
    test_openai_rate_limit_scenarios();
    test_non_rate_limit_errors();
    
    println!("\n=== Test Complete ===");
    println!("This demonstrates the new rate limit error detection functionality.");
    println!("In your application, you can now match on Error::RateLimit to handle rate limits specifically.");
}