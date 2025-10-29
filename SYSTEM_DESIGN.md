# GitPulse - System Design

## Overview

An agent that helps developers discover trending GitHub repositories through natural language queries and automated daily updates. This agent combines GitHub's Search API with LLM-based query parsing to provide flexible, context-aware repository discovery.

---

## System Architecture

### Components

1. **HTTP Server**
   - A2A protocol endpoint
   - Health check endpoint

2. **GitHub Client**
   - Search API integration
   - Query builder
   - Response parser

3. **LLM Client**
   - Query parser
   - Structured output extraction

4. **Cache Layer**
   - In-memory store
   - 6-hour TTL per query
   - Thread-safe operations

5. **Scheduler**
   - Cron jobs for proactive messages
   - Cache warming tasks

---

## Request Flow

### Reactive Path

```
User Query → External source → A2A Request  →  Agent
                                                 ↓
                                            Check Cache (LLM) or Parse with LLM (on cache miss)
                                                 ↓
                                            Check Cache (GitHub) or GitHub API Query (on cache miss)
                                                 ↓
                                            Format Response
                                                 ↓
                                            Return to initial source
```

### Proactive Path

```
Cron Trigger → Generate Query  →  GitHub API Query
                                      ↓
                                Format Message
                                      ↓
                                Webhook to External Channel
```

---

## A2A Protocol Integration

### Request Format

External source sends:
```json
{
  "jsonrpc": "2.0",
  "id": "request-id",
  "method": "message/send",
  "params": {
    "message": {
      "kind": "message",
      "role": "user",
      "parts": [{"kind": "text", "text": "query"}],
      "messageId": "message-id",
      "taskId": "task-id"
    },
    "configuration": {
      "blocking": true
    }
  }
}
```


### Response Format

Agent returns:
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
      "timestamp": "2025-10-26T10:30:00.000Z",
      "message": {
        "kind": "message",
        "messageId": "message-id",
        "role": "agent",
        "parts": [{"kind": "text", "text": "formatted response"}],
      }
    },
    "artifacts": [],
    "history": []
  },
  "error": {}
}
```

### Error Response Format

All errors return structured messages:
```json
{
  "jsonrpc": "2.0",
  "id": "request-id",
  "error": {
    "code": -32000,
    "message": "GitHub API unavailable",
    "data": {
      "suggestion": "Try again in a few minutes"
    }
  },
  "result": {}
}
```

---

## Project Structure

```
gitPulse/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── api/
│   │   ├── mod.rs
│   │   ├── routes.rs           
│   │   └── state.rs         
│   ├── config/
│   │   ├── mod.rs
│   │   ├── logging.rs           
│   │   └── settings.rs   
|   ├── models/
│   │   ├── mod.rs
|   |   ├── mod.rs
│   |   ├── a2a.rs        
│   |   ├── repository.rs        
│   |   └── query.rs          
│   ├── services/
│   │   ├── mod.rs
│   │   ├── ai.rs       
│   │   ├── cache.rs       
│   │   ├── github.rs       
│   │   ├── scheduler.rs
│   │   └── utils.rs      
│   └── utils/
│       ├── mod.rs
│       ├── helpers.rs
│       └── tasks.rs       
├── tests/
├── Dockerfile
└── Cargo.toml
```

---

## Dependencies

Core libraries:
- **anyhow**: Error handling
- **axum**: HTTP server
- **chrono**: Date/time handling
- **dashmap**: Thread-safe cache
- **dotenv** and **envy**: Environmental variable loading
- **google-ai-rs**: LLM query parsing
- **reqwest**: HTTP client
- **serde** and **serde_json**: Serialization
- **tokio**: Async runtime
- **tokio-cron-scheduler**: Job scheduling
- **tokio-test** and **tower**: Testing
- **tracing**, **tracing-appender**, and **tracing-subscriber**: Logging
- **utoipa** and **utoipa-swagger-ui**: Openapi/swagger docs
- **uuid**: Unique identifier

---

## Deployment Considerations

### Environment Variables

```
GITHUB_ACCESS_TOKEN=ghp_xxx
GITHUB_SEARCH_URL=https://api.github.com/search/repositories
LLM_API_KEY=xxx
LLM_MODEL=xxx
EXTERNAL_WEBHOOK_URL=https://telex.im/webhooks/xxx
CACHE_TTL=21600
HOST=0.0.0.0
PORT=8000
RUST_LOG=info
```

### Resource Requirements

- Memory: 512MB minimum for cache
- CPU: Minimal, I/O bound operations
- Network: Stable connection for API calls

### Monitoring

Track:
- GitHub API rate limit remaining
- Cache hit/miss ratio
- Average response time
- Failed queries

---

## Summary

The system design follows these principles:

- Simple architecture with clear component boundaries
- Aggressive caching to respect API rate limits
- Graceful degradation when services unavailable
- A2A protocol compliance for Telex integration
- Observable through structured logging
- Testable at each layer

The design prioritizes reliability and maintainability over complexity, ensuring the agent remains functional even when individual components fail.