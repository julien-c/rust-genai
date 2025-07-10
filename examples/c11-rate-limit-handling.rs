use genai::{Error, ModelIden};
use genai::adapter::AdapterKind;

fn main() {
    // This example demonstrates how to check for rate limit errors
    // In real usage, this would come from an actual API call that hits a rate limit
    
    let model_iden = ModelIden::new(AdapterKind::Anthropic, "claude-3-5-haiku-latest");
    
    // Example 1: Detect rate limit based on HTTP status code 429
    println!("Example 1: HTTP 429 rate limit detection");
    let error_429 = Error::RateLimit { 
        model_iden: model_iden.clone() 
    };
    
    match error_429 {
        Error::RateLimit { model_iden } => {
            println!("✓ Rate limit detected for model: {}", model_iden);
            println!("  Recommended action: Wait and retry after some time");
        }
        _ => println!("✗ Not a rate limit error"),
    }
    
    // Example 2: Check if an error is a rate limit error
    println!("\nExample 2: Error pattern matching");
    let error_normal = Error::RequiresApiKey { 
        model_iden: model_iden.clone() 
    };
    
    match error_normal {
        Error::RateLimit { .. } => {
            println!("✗ This should not be a rate limit error");
        }
        _ => println!("✓ This is not a rate limit error"),
    }
    
    println!("\nExample 3: Handling rate limits in application code");
    println!("```rust");
    println!("match client.exec_chat(model, chat_req, None).await {{");
    println!("    Ok(response) => {{");
    println!("        // Process successful response");
    println!("    }}");
    println!("    Err(Error::RateLimit {{ model_iden }}) => {{");
    println!("        eprintln!(\"Rate limit hit for {{}}, sleeping...\", model_iden);");
    println!("        tokio::time::sleep(Duration::from_secs(60)).await;");
    println!("        // Retry the request");
    println!("    }}");
    println!("    Err(other_error) => {{");
    println!("        eprintln!(\"Other error: {{}}\", other_error);");
    println!("    }}");
    println!("}}");
    println!("```");
}