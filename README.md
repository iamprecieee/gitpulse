# GitPulse

**An A2A-compatible AI agent that discovers trending GitHub repositories through natural language queries.**

GitPulse transforms GitHub repository discovery by allowing developers to query trending projects using natural language instead of complex search syntax. Built with Rust and fully compatible with the Agent-to-Agent (A2A) protocol, it integrates seamlessly with workflow automation platforms like Telex.

## Features

- **Natural Language Queries** - Ask questions like "What's trending in Rust?" or "Show me AI projects from this week"
- **LLM-Powered Parsing** - Uses Gemini or Claude to extract structured parameters from user queries
- **Smart Caching** - Aggressive caching strategy to respect GitHub API rate limits (6-hour TTL)
- **Proactive Updates** - Scheduled daily and weekly digests of trending repositories
- **A2A Protocol Compliant** - Fully compatible with Agent-to-Agent communication standards
- **High Performance** - Built with Rust for speed and reliability
- **Flexible Filtering** - Filter by language, topics, timeframe, and minimum stars

## Architecture

```
User Query → A2A Request   →  Agent
                                ↓
                            Check Cache (LLM)
                                ↓
                            Parse with LLM (if cache miss)
                                ↓
                            Check Cache (GitHub)
                                ↓
                            GitHub API Query (if cache miss)
                                ↓
                            Format Response
                                ↓
                            Return A2A Response
```

## Prerequisites

- **Rust** - For building and running the application
- **GitHub Personal Access Token** - Required for API authentication (5000 req/hour vs 60 unauthenticated)
- **LLM API Key** - Either Gemini or Claude API key
- **Docker** (optional) - For containerized deployment

## Quick Start

### 1. Clone the Repository

```bash
git clone <repository-url>
cd gitpulse
```

### 2. Configure Environment Variables

Create a `.env` file in the project root:

```bash
# LLM Configuration
LLM_PROVIDER=gemini  # or "anthropic" for Claude
LLM_API_KEY=your_llm_api_key
LLM_MODEL=gemini-2.5-flash  # or claude model name

# GitHub Configuration
GITHUB_ACCESS_TOKEN=ghp_your_github_token
GITHUB_SEARCH_URL=https://api.github.com/search/repositories

# External Webhook (for proactive messages)
EXTERNAL_WEBHOOK_URL=https://telex.im/webhooks/your_webhook

# Server Configuration
HOST=0.0.0.0
PORT=8000
RUST_LOG=info
CORS_ALLOWED_ORIGINS=http://localhost:3000

# Cache Configuration
CACHE_TTL=21600  # 6 hours in seconds
```

### 3. Build and Run

**Using Cargo (Development):**

```bash
# Build the project
cargo build --release

# Run the application
cargo run --release
```

**Using Docker:**

```bash
# Build the Docker image
docker build -t gitpulse .

# Run the container
docker run -p 8000:8000 --env-file .env gitpulse
```

The server will start on `http://localhost:8000` (or your configured host/port).

## API Endpoints

### Health Check

```bash
GET /health
```

Returns the health status of the service.

**Response:**
```json
{
  "status": "OK"
}
```

### Trending Repositories (A2A Endpoint)

```bash
POST /trending
```

Processes A2A-compliant requests for trending repositories.

**Request Example:**
```json
{
  "jsonrpc": "2.0",
  "id": "request-id",
  "method": "message/send",
  "params": {
    "message": {
      "kind": "message",
      "role": "user",
      "parts": [
        {
          "kind": "text",
          "text": "What's trending in Rust?"
        }
      ],
      "messageId": "message-id",
      "taskId": "task-id"
    },
    "configuration": {
      "blocking": true
    }
  }
}
```

**Response Example:**
```json
{
  "jsonrpc": "2.0",
  "id": "request-id",
  "result": {
    "kind": "task",
    "id": "task-id",
    "contextId": "context-id",
    "status": {
      "state": "completed",
      "timestamp": "2025-10-30T10:30:00.000Z",
      "message": {
        "kind": "message",
        "messageId": "response-id",
        "role": "agent",
        "parts": [
          {
            "kind": "text",
            "text": "Trending on GitHub (recent [week])\n\n1. rust-lang/rust - 150000 stars\n   Rust - The Rust programming language\n   https://github.com/rust-lang/rust\n\n..."
          }
        ]
      }
    },
    "artifacts": [...],
    "history": [...]
  }
}
```

### API Documentation

Swagger UI is available at:
```
http://localhost:8000/swagger-ui
```

## Query Examples

GitPulse supports various natural language queries:

- **General**: "What's trending on GitHub?"
- **Language-specific**: "Trending Rust projects"
- **Topic-based**: "What's hot in AI this week?"
- **Time-filtered**: "Top repos from last month"
- **Combined**: "Trending Rust web frameworks with over 100 stars"

The LLM extracts structured parameters:
- `language`: Programming language (e.g., "rust", "python")
- `topics`: List of keywords (e.g., ["machine-learning", "ai"])
- `timeframe`: "day", "week", "month", "quarter", or "year"
- `count`: Number of results (default: 5, max: 20)
- `min_stars`: Minimum star threshold (default: 10)

## Proactive Features

GitPulse includes scheduled jobs that automatically send trending repository updates:

- **Daily Digest** (9 AM): Top 5 trending repositories from yesterday
- **Weekly Roundup** (Monday 9 AM): Last week's most starred repositories

These are sent to the configured `EXTERNAL_WEBHOOK_URL` as A2A-compliant messages.

## Caching Strategy

GitPulse implements a two-tier caching system:

1. **LLM Query Cache** - Caches parsed query parameters to avoid repeated LLM calls
2. **GitHub Results Cache** - Caches repository search results to minimize API calls

Both caches use a configurable TTL (default: 6 hours) to balance freshness with API rate limits.

## Error Handling

The service gracefully handles various error scenarios:

- **GitHub API failures**: Returns cached results if available
- **Rate limit exceeded**: Falls back to cached data
- **LLM parsing errors**: Uses default parameters and continues
- **Invalid queries**: Returns structured error responses

## Testing

Run the test suite:

```bash
cargo test
```

The project includes tests for:
- A2A protocol parsing
- Query parameter extraction
- Client search functionality

## Project Structure

```
gitpulse/
├── src/
│   ├── api/              # HTTP routes and A2A handlers
│   ├── config/           # Configuration management
│   ├── models/           # Data models (A2A, Query, Repository)
│   ├── services/         # Core services (AI, GitHub, Cache, Scheduler)
│   └── utils/            # Helper functions and tasks
├── tests/                # Integration tests
├── logs/                 # Application logs
├── system_prompt.txt     # LLM system prompt
├── Dockerfile            # Docker configuration
└── Cargo.toml            # Rust dependencies
```

## Dependencies

Key dependencies:
- **axum** - HTTP server framework
- **reqwest** - HTTP client for GitHub API
- **google-ai-rs** / **anthropic-sdk-rust** - LLM clients
- **tokio** - Async runtime
- **dashmap** - Thread-safe caching
- **tokio-cron-scheduler** - Job scheduling
- **serde** / **serde_json** - Serialization

## Deployment

### Docker Deployment

The included `Dockerfile` builds a minimal distroless image:

```bash
docker build -t gitpulse:latest .
docker run -d -p 8000:8000 --env-file .env gitpulse:latest
```

## Integration with Telex

To integrate GitPulse with Telex workflows, create an AI Co-Worker and configure it with:

```json
{
  "active": false,
  "category": "utilities",
  "description": "",
  "id": "",
  "name": "git_pulse",
  "long_description": "",
  "short_description": "",
  "nodes": [
    {
      "id": "gitpulse_agent",
      "name": "GitPulse Agent",
      "parameters": {},
      "position": [
        816,
        -112
      ],
      "type": "a2a/generic-a2a-node",
      "typeVersion": 1,
      "url": "http://your-gitpulse-server/trending"
    }
  ],
  "pinData": {},
  "settings": {
    "executionOrder": "v1"
  }
}
```

## Limitations

- "Trending" is approximated using GitHub's search API, not official trending data
- Personal access token required for reasonable rate limits (5000 req/hour)
- Very new repositories (hours old) may not appear in results
- GitHub search is limited to 1000 results per query
- **Date parsing**: Currently does not support specific dates like "23, January 2013" - only relative timeframes (day, week, month, quarter, year) are supported. Specific date parsing is planned for future updates
