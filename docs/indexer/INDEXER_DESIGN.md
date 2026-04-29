# Indexer Integration Design

## Overview

The Linkora social contract emits `PostCreatedEvent` when posts are created on-chain. An off-chain indexer service monitors these events and builds a searchable index of post content to enable keyword search functionality.

## Architecture

```
Stellar Network → Indexer Service → Search Database → Web Frontend
     ↓               ↓                    ↓              ↓
PostCreatedEvent → Event Processing → Indexed Content → Search API
```

## Event Processing Flow

1. **Event Monitoring**: Indexer subscribes to contract events via Stellar RPC
2. **Event Parsing**: Extract post data from `PostCreatedEvent` 
3. **Content Indexing**: Store post content with full-text search capabilities
4. **API Serving**: Provide search endpoint for frontend queries

## PostCreatedEvent Structure

The contract emits events with this structure:
```rust
pub struct PostCreatedEvent {
    pub id: u64,
    pub author: Address,
}
```

## Required Indexer Components

### Event Subscriber
- Monitor Stellar network for contract events
- Parse `PostCreatedEvent` data
- Fetch full post content using `get_post(id)` contract call

### Search Index
- Full-text search engine (e.g., Elasticsearch, PostgreSQL with tsvector)
- Index post content for keyword matching
- Support pagination and relevance scoring

### API Server
- REST endpoint for search queries
- Rate limiting and input validation
- CORS configuration for web frontend

## Implementation Considerations

- **Event Reliability**: Handle network interruptions and missed events
- **Content Updates**: Posts can be deleted via `delete_post()` - indexer must handle removal
- **Performance**: Implement caching and efficient search algorithms
- **Security**: Validate and sanitize search queries to prevent injection attacks

## Integration Points

The web frontend integrates with the indexer via the search API defined in `API.md`. The indexer operates independently of the web application and can be deployed as a separate service.
