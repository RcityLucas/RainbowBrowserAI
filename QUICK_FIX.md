# Quick Fix for "go to stackoverflow" Issue

## Problem
When you type "go to stackoverflow and take screenshot", the system incorrectly navigates to `https://www.go.com` instead of `stackoverflow.com`.

## Solution
Edit the file `poc/src/llm_service.rs` around line 552-615:

### Find this code block:
```rust
} else if input_lower.contains("navigate") || input_lower.contains("go to") || input_lower.contains("open") || input_lower.contains("visit") || input_lower.contains("browse") {
    command.action = "navigate".to_string();
    command.confidence = 0.95;
    
    // Extract URL
    let url_regex = Regex::new(r"([a-zA-Z0-9][a-zA-Z0-9-]*\.)+[a-zA-Z]{2,}|[a-zA-Z0-9]+").unwrap();
    if let Some(cap) = url_regex.captures(&input_lower) {
        let mut url = cap[0].to_string();
        // Add .com if it's just a word
        if !url.contains('.') {
            url.push_str(".com");
        }
        command.url = Some(url);
    }
```

### Replace it with:
```rust
} else if input_lower.contains("navigate") || input_lower.contains("go to") || input_lower.contains("open") || input_lower.contains("visit") || input_lower.contains("browse") {
    command.action = "navigate".to_string();
    command.confidence = 0.95;
    
    // First try to extract a complete URL with domain extension
    let complete_url_regex = Regex::new(r"([a-zA-Z0-9][a-zA-Z0-9-]*\.)+[a-zA-Z]{2,}").unwrap();
    if let Some(cap) = complete_url_regex.captures(&input_lower) {
        command.url = Some(cap[0].to_string());
    } else {
        // If no complete URL, look for known websites or domain names
        // Skip common command words when looking for domain names
        let filtered_input = input_lower
            .replace("navigate to", "")
            .replace("go to", "")
            .replace("open", "")
            .replace("visit", "")
            .replace("browse to", "")
            .replace(" and ", " ")
            .replace(" take ", " ")
            .replace(" screenshot", "")
            .replace(" the ", " ")
            .replace(" a ", " ")
            .replace(" to ", " ");
        
        // Look for common website names first
        let known_sites = [
            ("stackoverflow", "stackoverflow.com"),
            ("google", "google.com"),
            ("github", "github.com"),
            ("youtube", "youtube.com"),
            ("reddit", "reddit.com"),
            ("twitter", "twitter.com"),
            ("facebook", "facebook.com"),
            ("amazon", "amazon.com"),
            ("wikipedia", "wikipedia.org"),
            ("linkedin", "linkedin.com"),
        ];
        
        let mut found_url = None;
        for (name, full_url) in &known_sites {
            if filtered_input.contains(name) {
                found_url = Some(full_url.to_string());
                break;
            }
        }
        
        // If no known site found, try generic domain extraction
        if found_url.is_none() {
            let domain_regex = Regex::new(r"\b([a-zA-Z][a-zA-Z0-9]{2,})\b").unwrap();
            if let Some(cap) = domain_regex.captures(&filtered_input) {
                let mut url = cap[1].to_string();
                // Add .com if it's just a word
                if !url.contains('.') {
                    url.push_str(".com");
                }
                found_url = Some(url);
            }
        }
        
        command.url = found_url;
    }
```

## How to Apply
1. Stop your server (Ctrl+C)
2. Edit `poc/src/llm_service.rs`
3. Make the above change
4. Run: `cargo build --release`
5. Start server: `RAINBOW_MOCK_MODE=true cargo run --release -- serve --port 3000`
6. Test: Navigate to http://localhost:3000/ and try "go to stackoverflow and take screenshot"

## Result
After the fix, "go to stackoverflow and take screenshot" will correctly navigate to `stackoverflow.com` instead of `go.com`.