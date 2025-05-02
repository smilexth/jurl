use anyhow::{Context, Result};
use clap::{Parser, ValueEnum, CommandFactory};
use colored::Colorize;
use reqwest::{
    blocking::{Client, Response},
    header::{HeaderMap, HeaderName, HeaderValue},
    Method,
};
use similar::{ChangeTag, TextDiff};
use std::{io::{self, Read}, str::FromStr, time::Instant, process};

/// jurl - A curl-like HTTP client in Rust
#[derive(Parser, Debug)]
#[clap(author, version, about)]
#[command(group = clap::ArgGroup::new("mode").required(false))]
#[command(after_help = "Example usage:
  jurl https://example.com
  jurl -m post -H \"Content-Type: application/json\" -d '{\"key\":\"value\"}' https://api.example.com
  jurl -D https://api1.example.com https://api2.example.com")]
struct Args {
    /// URL to send the request to
    #[clap(index = 1, required = false)]
    url: Option<String>,

    /// Second URL to compare (for diff mode)
    #[clap(index = 2)]
    url2: Option<String>,

    /// HTTP method to use
    #[clap(short, long, value_enum, default_value = "get")]
    method: HttpMethod,

    /// HTTP headers (format: "Name: Value")
    #[clap(short = 'H', long)]
    headers: Vec<String>,

    /// Request body
    #[clap(short, long)]
    data: Option<String>,

    /// Content type header
    #[clap(short = 'c', long)]
    content_type: Option<String>,

    /// Read request body from stdin
    #[clap(short = 's', long)]
    data_stdin: bool,

    /// Follow redirects
    #[clap(short = 'L', long)]
    follow: bool,

    /// Output format
    #[clap(short = 'o', long, value_enum, default_value = "default")]
    output: OutputFormat,

    /// Include response headers in the output
    #[clap(short = 'i', long)]
    include: bool,

    /// Output verbose information
    #[clap(short, long)]
    verbose: bool,
    
    /// Enable diff mode to compare responses between two URLs
    #[clap(short = 'D', long, group = "mode")]
    diff: bool,
    
    /// Only show differences between responses (instead of full diff)
    #[clap(long)]
    diff_only_changes: bool,
    
    /// Ignore whitespace in diff comparison
    #[clap(long)]
    diff_ignore_whitespace: bool,
}

#[derive(Debug, Copy, Clone, ValueEnum, PartialEq)]
enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Head => write!(f, "HEAD"),
            HttpMethod::Options => write!(f, "OPTIONS"),
        }
    }
}

#[derive(Debug, Copy, Clone, ValueEnum, PartialEq)]
enum OutputFormat {
    Default,
    Json,
    Headers,
}

impl From<HttpMethod> for Method {
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::Get => Method::GET,
            HttpMethod::Post => Method::POST,
            HttpMethod::Put => Method::PUT,
            HttpMethod::Delete => Method::DELETE,
            HttpMethod::Patch => Method::PATCH,
            HttpMethod::Head => Method::HEAD,
            HttpMethod::Options => Method::OPTIONS,
        }
    }
}

fn parse_headers(headers: &[String]) -> Result<HeaderMap> {
    let mut header_map = HeaderMap::new();
    for header in headers {
        if let Some((name, value)) = header.split_once(':') {
            let name = name.trim();
            let value = value.trim();
            
            let header_name = HeaderName::from_str(name)
                .with_context(|| format!("Invalid header name: {}", name))?;
            let header_value = HeaderValue::from_str(value)
                .with_context(|| format!("Invalid header value: {}", value))?;
            
            header_map.insert(header_name, header_value);
        } else {
            anyhow::bail!("Invalid header format: {}", header);
        }
    }
    Ok(header_map)
}

fn format_response(response: Response, args: &Args, url: &str) -> Result<String> {
    let status = response.status();
    let response_headers = response.headers().clone();
    let start = Instant::now();
    let body = response.text()?;
    let duration = start.elapsed();

    if args.verbose {
        println!("{} {} {}", "-->".green(), args.method.to_string().blue(), url);
        println!("{} {} {}", "<--".green(), status.as_str().blue(), status.canonical_reason().unwrap_or(""));
        println!("{} {:.2?}", "Time:".green(), duration);
    }

    if args.include || args.output == OutputFormat::Headers {
        println!("{}", "Response Headers:".green());
        for (name, value) in response_headers.iter() {
            println!("{}: {}", name.to_string().yellow(), value.to_str().unwrap_or(""));
        }
        println!();
    }

    let formatted_body = match args.output {
        OutputFormat::Default => body.clone(),
        OutputFormat::Json => {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&body) {
                serde_json::to_string_pretty(&parsed)?
            } else {
                body.clone()
            }
        }
        OutputFormat::Headers => body.clone(), // Headers already printed above
    };

    // Only print the body if we're not in diff mode
    if !args.diff || args.url2.is_none() {
        println!("{}", formatted_body);
    }

    Ok(formatted_body)
}

/// Compare two responses and display their differences
fn compare_responses(body1: &str, body2: &str, args: &Args) -> Result<()> {
    // Prepare the texts for comparison
    let text1 = if args.diff_ignore_whitespace {
        body1.split_whitespace().collect::<Vec<_>>().join(" ")
    } else {
        body1.to_string()
    };
    
    let text2 = if args.diff_ignore_whitespace {
        body2.split_whitespace().collect::<Vec<_>>().join(" ")
    } else {
        body2.to_string()
    };
    
    // Create the diff
    let diff = TextDiff::from_lines(&text1, &text2);
    
    println!("{}", "Diff Results:".green().bold());
    println!("{}", "=".repeat(50).yellow());
    
    let mut has_changes = false;
    
    for change in diff.iter_all_changes() {
        let (sign, color) = match change.tag() {
            ChangeTag::Delete => ("-", colored::Color::Red),
            ChangeTag::Insert => ("+", colored::Color::Green),
            ChangeTag::Equal => {
                if args.diff_only_changes {
                    continue; // Skip unchanged lines if only showing changes
                }
                (" ", colored::Color::White)
            }
        };
        
        has_changes = has_changes || change.tag() != ChangeTag::Equal;
        
        let line = format!("{} {}", sign, change.value());
        println!("{}", line.color(color));
    }
    
    if !has_changes {
        println!("{}", "No differences found.".bright_green());
    }
    
    println!("{}", "=".repeat(50).yellow());
    Ok(())
}

fn main() -> Result<()> {
    let mut app = Args::command();
    let mut args = Args::parse();
    
    // Show usage if no URL is provided and it's not help mode
    if args.url.is_none() {
        app.print_help().unwrap();
        println!("\n\nError: The URL argument is required");
        process::exit(1);
    }
    
    let url = args.url.as_ref().unwrap();
    
    // Validate that if diff mode is enabled, a second URL is provided
    if args.diff && args.url2.is_none() {
        anyhow::bail!("Diff mode requires a second URL to compare with");
    }
    
    let mut headers = parse_headers(&args.headers)?;
    
    // Add content-type if provided
    if let Some(content_type) = &args.content_type {
        headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_str(content_type)?,
        );
    }
    
    let client = Client::builder()
        .default_headers(headers.clone())  // Clone for potential second request
        .redirect(if args.follow { reqwest::redirect::Policy::limited(10) } else { reqwest::redirect::Policy::none() })
        .build()?;
    
    let method: Method = args.method.into();
    let mut request = client.request(method.clone(), url);
    
    // Handle request body
    let body = if args.data_stdin {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        Some(buffer)
    } else {
        args.data.take()  // Use take() to move out of the Option without consuming args
    };
    
    if let Some(data) = body {
        request = request.body(data);
    }
    
    let response = request.send()
        .with_context(|| format!("Failed to send request to {}", url))?;
    
    let formatted_response = format_response(response, &args, url)?;
    
    // If in diff mode and a second URL is provided, fetch it and compare
    if args.diff && args.url2.is_some() {
        let url2 = args.url2.as_ref().unwrap();
        
        if args.verbose {
            println!("\n{} {} {}", "-->".green(), args.method.to_string().blue(), url2);
        }
        
        let request2 = client.request(method, url2);
        let response2 = request2.send()
            .with_context(|| format!("Failed to send request to {}", url2))?;
        
        if args.verbose {
            let status = response2.status();
            println!("{} {} {}", "<--".green(), status.as_str().blue(), status.canonical_reason().unwrap_or(""));
        }
        
        let formatted_response2 = format_response(response2, &args, url2)?;
        
        // Compare and display the differences
        compare_responses(&formatted_response, &formatted_response2, &args)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_headers() {
        let headers = vec![
            "Content-Type: application/json".to_string(),
            "Authorization: Bearer token123".to_string(),
        ];
        
        let header_map = parse_headers(&headers).unwrap();
        assert_eq!(header_map.len(), 2);
        assert_eq!(
            header_map.get("content-type").unwrap().to_str().unwrap(),
            "application/json"
        );
        assert_eq!(
            header_map.get("authorization").unwrap().to_str().unwrap(),
            "Bearer token123"
        );
    }
    
    #[test]
    fn test_invalid_header_format() {
        let headers = vec!["InvalidHeader".to_string()];
        assert!(parse_headers(&headers).is_err());
    }
    
    #[test]
    fn test_http_method_conversion() {
        assert_eq!(Method::from(HttpMethod::Get), Method::GET);
        assert_eq!(Method::from(HttpMethod::Post), Method::POST);
        assert_eq!(Method::from(HttpMethod::Put), Method::PUT);
        assert_eq!(Method::from(HttpMethod::Delete), Method::DELETE);
    }
}