# ZK-Persona Database Implementation

This document describes the comprehensive database implementation for the ZK-Persona system, including schema design, model structs, connection pooling, and CRUD operations with repository patterns.

## 🗄️ Database Schema Design

### Core Tables Implemented

1. **`users`** - Core user management
2. **`behavior_sessions`** - Session grouping for behavior data
3. **`behavior_inputs`** - User behavior data (enhanced from existing)
4. **`scoring_results`** - AI scoring results (existing)
5. **`zk_proofs`** - Enhanced ZK proof management
6. **`reputation_records`** - Time-series reputation tracking

### Migration Files

- **`0007_zkpersona_extended_schema.sql`** - Extended schema migration that adds:
  - Users table with wallet integration
  - Behavior sessions for grouping inputs
  - Enhanced behavior_inputs with new fields
  - Comprehensive zk_proofs table
  - Reputation records with scoring categories

### Key Features

- **UUID Primary Keys** for all entities
- **Comprehensive Indexing** for performance
- **Foreign Key Relationships** with proper cascade rules
- **JSONB Fields** for flexible metadata storage
- **Timestamp Tracking** (cid, ctime, mid, mtime pattern)
- **Enum Constraints** for data integrity
- **Check Constraints** for business rule validation

## 🏗️ Model Structs & Domain Objects

### Location: `crates/shared/jd_domain/src/zkpersona_domain/models.rs`

#### Core Models
- `User` - User entity with privacy settings
- `BehaviorSession` - Session management
- `BehaviorInput` - Behavior data with type classification
- `ScoringResult` - AI scoring output
- `ZkProof` - Comprehensive ZK proof data
- `ReputationRecord` - Reputation tracking over time

#### Enums for Type Safety
- `UserStatus`, `SessionType`, `SessionStatus`
- `InputType`, `InputSource`
- `ProofType`, `ProofProtocol`, `VerificationStatus`
- `ScoringCategory`, `ScoringPeriod`, `ReputationStatus`

#### DTOs & Filters
- `CreateUserRequest`, `CreateBehaviorInputRequest`, etc.
- `BehaviorInputFilter`, `ReputationFilter`, `ZkProofFilter`
- `PaginationParams` with sorting support
- `UserReputationSummary`, `BehaviorAnalyticsSummary`

#### Error Handling
- `ZkPersonaError` enum with comprehensive error types
- Integration with existing error handling patterns

## 🔧 Database Connection & Configuration

### Location: `crates/infrastructure/jd_storage/src/config.rs`

#### Features Implemented
- **Environment-based Configuration** via `DatabaseConfig::from_env()`
- **Connection Pooling** with configurable limits
- **Health Checks** and monitoring
- **Retry Logic** for connection failures
- **Auto-migration** support
- **Transaction Management** integration

#### Configuration Options
```rust
pub struct DatabaseConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
    pub idle_timeout_secs: Option<u64>,
    pub max_lifetime_secs: Option<u64>,
    pub ssl_mode: String,
    pub auto_migrate: bool,
    pub enable_transactions: bool,
    pub test_connection: bool,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
}
```

#### Environment Variables
- `DATABASE_URL` or `POSTGRES_URL` - Database connection string
- `DB_MAX_CONNECTIONS` - Maximum pool connections (default: 50)
- `DB_MIN_CONNECTIONS` - Minimum pool connections (default: 5)
- `DB_CONNECT_TIMEOUT_SECS` - Connection timeout (default: 30)
- `DB_IDLE_TIMEOUT_SECS` - Idle connection timeout
- `DB_MAX_LIFETIME_SECS` - Connection max lifetime
- `DB_SSL_MODE` - SSL mode (default: prefer)
- `DB_AUTO_MIGRATE` - Run migrations on startup (default: true)
- `DB_ENABLE_TRANSACTIONS` - Enable transaction support (default: true)
- `DB_TEST_CONNECTION` - Test connection on startup (default: true)
- `DB_RETRY_ATTEMPTS` - Connection retry attempts (default: 3)
- `DB_RETRY_DELAY_MS` - Retry delay in milliseconds (default: 1000)

## 📊 Repository Pattern Implementation

### Location: `crates/infrastructure/jd_storage/src/repository/behavior_input_repository.rs`

#### Features Implemented

##### Core CRUD Operations
- `find_by_id()` - Find single entity by ID
- `find_all()` - Find all entities
- `save()` - Create new entity
- `update()` - Update existing entity
- `delete()` - Delete entity by ID
- `count()` - Count total entities

##### Advanced Filtering
- `find_by_filter()` - Filter by multiple criteria
- `count_by_filter()` - Count filtered results
- `delete_by_filter()` - Bulk delete by filter
- `find_by_user_id()` - Find by user
- `find_by_session_id()` - Find by session
- `find_unprocessed()` - Find unprocessed inputs

##### Pagination Support
- `find_paginated()` - Basic pagination
- `find_by_filter_paginated()` - Filtered pagination
- Configurable sorting and ordering
- Complete pagination metadata

##### Specialized Methods
- `create_from_request()` - Create from DTO
- `mark_as_processed()` - Update processing status
- `get_analytics_summary()` - Generate analytics

#### Filter Capabilities
```rust
pub struct BehaviorInputFilter {
    pub user_id: Option<Id>,
    pub behavior_session_id: Option<Id>,
    pub session_id: Option<String>,
    pub input_type: Option<InputType>,
    pub source: Option<InputSource>,
    pub processed: Option<bool>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}
```

#### Pagination Parameters
```rust
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<SortOrder>,
}
```

## 🚀 Usage Examples

### Complete Example: `examples/zkpersona_database_usage.rs`

The comprehensive example demonstrates:
1. **Database Initialization** with configuration
2. **Creating Behavior Inputs** with different types
3. **Reading Operations** (by ID, by user, counting)
4. **Filtering Operations** (by type, status, date range)
5. **Pagination** with configurable page size
6. **Update Operations** (marking as processed)
7. **Analytics** summary generation
8. **Advanced Filtering** with pagination
9. **Health Checks** and monitoring
10. **Cleanup Operations** for testing

### Quick Start

```rust
use jd_storage::{
    config::{DatabaseConfig, DatabaseManager},
    repository::{BehaviorInputRepository, Repository},
};

// Initialize database
let config = DatabaseConfig::from_env()?;
let mut db_manager = DatabaseManager::new(config)?;
db_manager.initialize().await?;

// Create repository
let dbx = db_manager.dbx()?;
let repo = BehaviorInputRepository::new(dbx);

// Use repository
let inputs = repo.find_all().await?;
```

## 🏛️ Architecture Integration

### Integration Points

1. **Base API Template** - Uses existing `jd_core::base` patterns
2. **Transaction Management** - Integrates with `jd_storage::dbx::Dbx`
3. **Error Handling** - Follows existing error patterns
4. **Domain Models** - Extends `jd_domain` structure
5. **Configuration** - Compatible with existing config system

### Repository Trait Implementation

The implementation follows the established repository traits:
- `Repository<T, ID>` - Basic CRUD
- `FilterableRepository<T, ID, F>` - Filtering support
- `PaginatedRepository<T, ID>` - Pagination support
- `FilterablePaginatedRepository<T, ID, F>` - Combined filtering + pagination

## 🔒 Security & Best Practices

### Implemented Security Features

1. **SQL Injection Prevention** - Using parameterized queries with sea-query
2. **Input Validation** - Enum constraints and check constraints
3. **Connection Security** - SSL support and secure connection pooling
4. **Error Handling** - No sensitive data exposure in errors
5. **Transaction Safety** - Proper transaction management with rollback

### Performance Optimizations

1. **Database Indexing** - Comprehensive indexes for common queries
2. **Connection Pooling** - Efficient connection reuse
3. **Pagination** - Limit large result sets
4. **Query Optimization** - Efficient SQL generation with sea-query
5. **Lazy Loading** - Optional fields and relationships

## 📈 Monitoring & Health Checks

### Health Check Features
- Database connectivity testing
- Connection pool statistics
- Query performance monitoring
- Migration status tracking

### Metrics Available
- Active/idle connection counts
- Query execution times
- Error rates and types
- Pool utilization statistics

## 🔄 Migration Strategy

### Migration Files Structure
```
sql/
├── 0001_initial.sql           # Base schema
├── 0002_user.sql             # User management
├── 0003_sponsored_transactions.sql
├── 0004_auth.sql             # Authentication
├── 0005_unified_auth_rbac.sql
├── 0006_zkpersona.sql        # Original ZK-Persona
└── 0007_zkpersona_extended_schema.sql  # Extended schema (NEW)
```

### Backward Compatibility
- Extends existing tables without breaking changes
- Adds new tables without affecting existing functionality
- Optional foreign keys to maintain flexibility

## 🧪 Testing

### Test Data Setup
The example includes creation of realistic test data with:
- Multiple input types (DeFi, NFT, Social)
- Different sources (Blockchain, API, Web)
- Various session configurations
- Processing status variations

### Cleanup Support
Environment variable `CLEANUP_TEST_DATA=true` enables automatic cleanup of test data.

## 📚 Dependencies Added

### New Dependencies in Cargo.toml files:

#### `jd_domain/Cargo.toml`
- `chrono` - Date/time handling
- `rust_decimal` - Precise decimal arithmetic
- `thiserror` - Error handling

#### `jd_storage/Cargo.toml`
- `sea-query` - Query builder
- `sea-query-binder` - SQLx integration
- `chrono` - Date/time handling
- `uuid` - UUID support
- `jd_domain` - Domain models

## 🎯 Next Steps

This implementation provides a solid foundation for the ZK-Persona system. Potential enhancements:

1. **Additional Repositories** - Implement repositories for other entities
2. **Caching Layer** - Add Redis caching for frequently accessed data
3. **Event Sourcing** - Track entity state changes
4. **Audit Logging** - Comprehensive audit trail
5. **Bulk Operations** - Batch insert/update capabilities
6. **Read Replicas** - Support for read-only replicas
7. **Sharding** - Horizontal scaling support

The implementation follows Rust best practices and integrates seamlessly with the existing codebase architecture.