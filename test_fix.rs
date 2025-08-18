// Test program to demonstrate the URL parsing fix
fn parse_command(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    
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
    
    println!("Original: '{}'", input);
    println!("Filtered: '{}'", filtered_input.trim());
    
    // Look for common website names first
    let known_sites = [
        ("stackoverflow", "stackoverflow.com"),
        ("google", "google.com"),
        ("github", "github.com"),
    ];
    
    for (name, full_url) in &known_sites {
        if filtered_input.contains(name) {
            println!("Found known site: {} -> {}", name, full_url);
            return Some(full_url.to_string());
        }
    }
    
    None
}

fn main() {
    let test_cases = vec![
        "go to stackoverflow and take screenshot",
        "navigate to google", 
        "open github.com",
        "go to www.go.com",
    ];
    
    for test in test_cases {
        println!("\n--- Testing: '{}' ---", test);
        if let Some(url) = parse_command(test) {
            println!("✅ Result: {}", url);
        } else {
            println!("❌ No URL found");
        }
    }
}