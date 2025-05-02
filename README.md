# jurl

A curl-like HTTP client written in Rust that provides a simple command-line interface for making HTTP requests, with unique features like response diffing.

## Features

- Support for all common HTTP methods (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS)
- Custom headers
- Request body from command line or stdin
- Follow redirects
- Pretty-print JSON responses
- Colored output
- Verbose mode for debugging
- Display response headers
- **Response Diff:** Compare responses between two URLs with colorized diff output (unique to jurl!)

## Installation

### From source

1. Clone the repository:
```sh
git clone https://github.com/yourusername/jurl.git
cd jurl
```

2. Build with Cargo:
```sh
cargo build --release
```

3. The binary will be available at `target/release/jurl`

4. Optionally, add to your PATH:
```sh
cp target/release/jurl /usr/local/bin/
```

### From crates.io (not yet available)

```sh
cargo install jurl
```

## Usage

```
jurl [OPTIONS] <URL>
```

### Examples

#### Basic GET request:
```sh
jurl https://httpbin.org/get
```

#### POST request with JSON body:
```sh
jurl -m post -H "Content-Type: application/json" -d '{"key": "value"}' https://httpbin.org/post
```

#### POST form data:
```sh
jurl -m post -c "application/x-www-form-urlencoded" -d "key1=value1&key2=value2" https://httpbin.org/post
```

#### Send data from a file:
```sh
cat data.json | jurl -m post -d -c "application/json" https://httpbin.org/post
```

#### Display response headers:
```sh
jurl -i https://httpbin.org/get
```

#### Follow redirects:
```sh
jurl -L https://httpbin.org/redirect/3
```

#### Pretty-print JSON response:
```sh
jurl -o json https://httpbin.org/json
```

#### Verbose output:
```sh
jurl -v https://httpbin.org/get
```

#### Compare responses between two URLs (diff mode):
```sh
jurl -D https://httpbin.org/get https://httpbin.org/anything
```

#### Show only differences between responses:
```sh
jurl -D --diff-only-changes https://api.example.com/v1/users https://api.example.com/v2/users
```

#### Ignore whitespace in diff comparison:
```sh
jurl -D --diff-ignore-whitespace https://api.example.com/data1 https://api.example.com/data2
```

## Command Line Options

```
Usage: jurl [OPTIONS] <URL> [URL2]

Arguments:
  <URL>   URL to send the request to
  [URL2]  Second URL to compare (required when using diff mode)

Options:
  -m, --method <METHOD>              HTTP method to use [default: get] [possible values: get, post, put, delete, patch, head, options]
  -H, --headers <HEADERS>...         HTTP headers (format: "Name: Value")
  -b, --data <DATA>                  Request body
  -c, --content-type <CONTENT_TYPE>  Content type header
  -d, --data-stdin                   Read request body from stdin
  -L, --follow                       Follow redirects
  -o, --output <OUTPUT>              Output format [default: default] [possible values: default, json, headers]
  -i, --include                      Include response headers in the output
  -v, --verbose                      Output verbose information
  -D, --diff                         Enable diff mode to compare responses between two URLs
      --diff-only-changes            Only show differences between responses (instead of full diff)
      --diff-ignore-whitespace       Ignore whitespace in diff comparison
  -h, --help                         Print help
  -V, --version                      Print version
```

## Features Comparison with cURL

| Feature                | jurl | curl |
|------------------------|------|------|
| HTTP/HTTPS requests    | ✅    | ✅    |
| Custom headers         | ✅    | ✅    |
| Request body           | ✅    | ✅    |
| Follow redirects       | ✅    | ✅    |
| JSON formatting        | ✅    | ❌    |
| Colored output         | ✅    | ❌    |
| Compare URL responses  | ✅    | ❌    |
| Diff highlighting      | ✅    | ❌    |
| Whitespace-aware diff  | ✅    | ❌    |
| FTPS/SCP/SFTP          | ❌    | ✅    |
| Proxy support          | ❌    | ✅    |
| Cookie handling        | ❌    | ✅    |
| Certificate validation | ❌    | ✅    |

## Development

### Running tests

```sh
cargo test
```

### Building locally

```sh
cargo build
```

## License

MIT License

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request