# GitPulse

## Problem Statement

GitHub's trending page exists but has no official API (at the time of creating this). Developers need a way to:
- Query trending repositories by language, topic, or timeframe
- Receive automated updates about popular projects
- Filter results using natural language instead of complex search syntax

---

## Core Functionality

### Reactive Features

Users interact with the agent through natural language queries:

- General queries: "What's trending on GitHub?"
- Language-specific: "Trending Rust projects"
- Topic-based: "What's hot in AI this week?"
- Time-filtered: "Top repos from last month"
- Combined filters: "Trending Rust web frameworks"

### Proactive Features

Scheduled automated messages:

- Daily (9 AM): Top 5 trending repositories from yesterday
- Weekly (Monday 9 AM): Last week's most starred repositories

---

## Technical Approach

### GitHub Search API Strategy

The official GitHub API search endpoint will be used to simulate trending:

```
GET https://api.github.com/search/repositories
```

Query construction follows this pattern:
- Base filter: `created:>DATE` (repos created after a date)
- Recent filter: `pushed:>DATE` (repos updated after a date)
- Language filter: `language:LANGUAGE`
- Topic filter: `topic:TOPIC` or keywords in search
- Sorting: `sort=stars&order=desc`

Example queries:
- Trending today: `created:>2025-10-26+stars:>50`
- Trending in AI: `ai+created:>2025-10-26+stars:>100`
- Rust repos: `language:rust+created:>2025-10-26+stars:>20`
- Chained qualifiers: `language:rust+language:python+created:>2025-10-26+stars:>20`

### Trending Formula

```
Trending = Recent (created in last X days) + Popular (high stars) + Active (recent updates)
```

### Rate Limits

- Unauthenticated: 60 requests/hour per IP
- With personal access token: 5000 requests/hour per user
- Solution: Use token-based auth and caching

---

## Query Parameter Support

Users can specify:
- **count**: Number of results (default: 5, max: 20)
- **timeframe**: day, week, month (default: week)
- **language**: Any GitHub language
- **topic**: Keywords or topics
- **min_stars**: Minimum star threshold

Examples:
- "Show me 10 trending Python repos"
- "What's trending in Rust today?"
- "Top AI projects from last month"

---

## LLM Query Parser

Use Gemini free API to parse natural language into structured parameters:

Input: "What's hot in machine learning this week?"

LLM extracts:
```json
{
  "keyword": "machine learning",
  "language": null,
  "timeframe": "week",
  "min_stars": 50
}
```

Agent converts to GitHub query:
```
machine+learning+created:>2025-10-29+stars:>50+pushed:>2025-10-29
```

---

## Data Model

### Repository Response Structure

```
TrendingRepo:
  - name: string (owner/repo)
  - description: string
  - url: string (GitHub link)
  - language: string or null
  - stars: integer
```

### User Response Format

Sample text response with formatted markdown:

```
Trending in AI This Week

1. anthropics/claude-code - 40,400 stars
   Python - Agentic coding tool that lives in your terminal
   github.com/anthropics/claude-code
   
2. openai/whisper - 90,000 stars
   Rust - Robust Speech Recognition via Large-Scale Weak Supervision
   github.com/openai/whisper

[3 more entries]
```

---

## Caching Strategy

### Cache Structure

```
Key: "trending:{language}:{topic}:{timeframe}"
Value: {
  repos: array of TrendingRepo,
  cached_at: timestamp,
  ttl: 6 hours
}
```

### Cache Invalidation

- Time-based: 6 hours
- Manual: Clear on demand if needed
- No cache: User explicitly requests fresh data

---

## Error Handling Strategy

### API Failures

- GitHub API down: Return cached results with stale warning
- Rate limit hit: Return cached results
- Invalid query: Ask user to rephrase

### LLM Failures

- Parsing error: Fall back to keyword extraction
- Timeout: Use basic query pattern
- Invalid output: Request clarification from user

---

## Limitations

### Known Constraints

1. "Trending" is approximated, not official GitHub data
2. Personal access token required for reasonable rate limits
3. Very new repos (hours old) may not appear
4. Search limited to 1000 results per query

---

## Summary

GitPulse provides a natural language interface to GitHub repository discovery through:
- Smart query parsing with LLM
- Efficient caching to respect rate limits
- Scheduled updates for passive discovery
- A2A protocol compliance for Telex integration

The system is designed to be simple, maintainable, and within free tier API limits while providing genuine value to developers looking for quality projects.