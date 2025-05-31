use chrono::Utc;
use serde_json::json;
use std::env;

use jd_storage::{
    config::{DatabaseConfig, DatabaseManager},
    repository::{BehaviorInputRepository, Repository, FilterableRepository, PaginatedRepository},
};
use jd_domain::{
    Id,
    zkpersona_domain::models::{
        CreateBehaviorInputRequest, BehaviorInputFilter, InputType, InputSource,
        PaginationParams, SortOrder
    }
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::init();

    // Load database configuration from centralized config system
    // Copy .env.dev to .env or set environment variables before running:
    // export POSTGRES_DSN="postgresql://username:password@localhost:5432/zkpersona"
    let config = DatabaseConfig::from_env()
        .unwrap_or_else(|_| {
            println!("⚠️  Using default database configuration. Copy .env.dev to .env for proper setup.");
            DatabaseConfig {
                database_url: "postgresql://postgres:password@localhost:5432/zkpersona".to_string(),
                ..Default::default()
            }
        });

    println!("🚀 Initializing ZK-Persona Database Example");
    println!("📊 Database URL: {}", mask_password(&config.database_url));

    // Initialize database manager
    let mut db_manager = DatabaseManager::new(config)?;
    db_manager.initialize().await?;

    // Get database pool and create Dbx instance
    let dbx = db_manager.dbx()?;
    
    // Create repository
    let repo = BehaviorInputRepository::new(dbx);

    println!("\n✅ Database connection established");

    // ============================================================================================
    // Example 1: Create Behavior Inputs
    // ============================================================================================
    println!("\n📝 Example 1: Creating Behavior Inputs");

    let user_id = Id::generate();
    println!("👤 Generated User ID: {}", user_id);

    // Create different types of behavior inputs
    let behavior_inputs = vec![
        CreateBehaviorInputRequest {
            user_id: Some(user_id.clone()),
            behavior_session_id: None,
            session_id: Some("session-001".to_string()),
            input_data: json!({
                "transactions": [
                    {"type": "swap", "amount": 1000, "timestamp": "2024-01-01T00:00:00Z"},
                    {"type": "stake", "amount": 500, "timestamp": "2024-01-02T00:00:00Z"}
                ],
                "interactions": {
                    "dao_votes": 3,
                    "nft_trades": 2,
                    "defi_protocols": ["uniswap", "aave", "compound"]
                }
            }),
            input_type: Some(InputType::Defi),
            source: Some(InputSource::Blockchain),
        },
        CreateBehaviorInputRequest {
            user_id: Some(user_id.clone()),
            behavior_session_id: None,
            session_id: Some("session-002".to_string()),
            input_data: json!({
                "nft_activities": [
                    {"action": "mint", "collection": "CryptoPunks", "token_id": 1234},
                    {"action": "trade", "collection": "BAYC", "price": 50.0}
                ],
                "marketplace_interactions": 5
            }),
            input_type: Some(InputType::Nft),
            source: Some(InputSource::Api),
        },
        CreateBehaviorInputRequest {
            user_id: Some(user_id.clone()),
            behavior_session_id: None,
            session_id: Some("session-003".to_string()),
            input_data: json!({
                "social_activities": {
                    "posts": 15,
                    "likes": 120,
                    "shares": 25,
                    "comments": 40
                },
                "engagement_score": 85.5
            }),
            input_type: Some(InputType::Social),
            source: Some(InputSource::Web),
        },
    ];

    let mut created_ids = Vec::new();
    for (i, request) in behavior_inputs.into_iter().enumerate() {
        match repo.create_from_request(request).await {
            Ok(behavior_input) => {
                println!("  ✅ Created behavior input {}: {}", i + 1, behavior_input.id);
                created_ids.push(behavior_input.id);
            }
            Err(e) => {
                println!("  ❌ Failed to create behavior input {}: {}", i + 1, e);
            }
        }
    }

    // ============================================================================================
    // Example 2: Read Operations
    // ============================================================================================
    println!("\n📖 Example 2: Reading Behavior Inputs");

    // Find by ID
    if let Some(first_id) = created_ids.first() {
        match repo.find_by_id(first_id.clone()).await {
            Ok(Some(behavior_input)) => {
                println!("  📄 Found behavior input: {} (Type: {:?})", 
                    behavior_input.id, behavior_input.input_type);
            }
            Ok(None) => println!("  ❌ Behavior input not found"),
            Err(e) => println!("  ❌ Error finding behavior input: {}", e),
        }
    }

    // Find by user ID
    match repo.find_by_user_id(user_id.clone()).await {
        Ok(inputs) => {
            println!("  👤 Found {} behavior inputs for user {}", inputs.len(), user_id);
            for input in &inputs {
                println!("    - {} ({:?}, {:?})", input.id, input.input_type, input.source);
            }
        }
        Err(e) => println!("  ❌ Error finding user inputs: {}", e),
    }

    // Count total inputs
    match repo.count().await {
        Ok(count) => println!("  📊 Total behavior inputs in database: {}", count),
        Err(e) => println!("  ❌ Error counting inputs: {}", e),
    }

    // ============================================================================================
    // Example 3: Filtering Operations
    // ============================================================================================
    println!("\n🔍 Example 3: Filtering Behavior Inputs");

    // Filter by input type
    let defi_filter = BehaviorInputFilter {
        input_type: Some(InputType::Defi),
        ..Default::default()
    };

    match repo.find_by_filter(defi_filter).await {
        Ok(defi_inputs) => {
            println!("  💰 Found {} DeFi behavior inputs", defi_inputs.len());
        }
        Err(e) => println!("  ❌ Error filtering DeFi inputs: {}", e),
    }

    // Filter by processed status
    let unprocessed_filter = BehaviorInputFilter {
        processed: Some(false),
        ..Default::default()
    };

    match repo.find_by_filter(unprocessed_filter).await {
        Ok(unprocessed_inputs) => {
            println!("  ⏳ Found {} unprocessed behavior inputs", unprocessed_inputs.len());
        }
        Err(e) => println!("  ❌ Error filtering unprocessed inputs: {}", e),
    }

    // Filter by date range
    let date_filter = BehaviorInputFilter {
        start_date: Some(Utc::now() - chrono::Duration::hours(1)),
        end_date: Some(Utc::now()),
        ..Default::default()
    };

    match repo.find_by_filter(date_filter).await {
        Ok(recent_inputs) => {
            println!("  📅 Found {} behavior inputs from the last hour", recent_inputs.len());
        }
        Err(e) => println!("  ❌ Error filtering by date: {}", e),
    }

    // ============================================================================================
    // Example 4: Pagination
    // ============================================================================================
    println!("\n📄 Example 4: Paginated Queries");

    match repo.find_paginated(1, 2).await {
        Ok(paginated_result) => {
            println!("  📑 Page 1 (2 per page):");
            println!("    - Total items: {}", paginated_result.total);
            println!("    - Total pages: {}", paginated_result.total_pages);
            println!("    - Items on this page: {}", paginated_result.items.len());
            
            for (i, input) in paginated_result.items.iter().enumerate() {
                println!("      {}. {} ({:?})", i + 1, input.id, input.input_type);
            }
        }
        Err(e) => println!("  ❌ Error with pagination: {}", e),
    }

    // ============================================================================================
    // Example 5: Update Operations
    // ============================================================================================
    println!("\n✏️  Example 5: Update Operations");

    // Mark some inputs as processed
    for (i, id) in created_ids.iter().take(2).enumerate() {
        match repo.mark_as_processed(id.clone()).await {
            Ok(true) => println!("  ✅ Marked behavior input {} as processed", i + 1),
            Ok(false) => println!("  ⚠️  Behavior input {} not found for processing", i + 1),
            Err(e) => println!("  ❌ Error marking input {} as processed: {}", i + 1, e),
        }
    }

    // Find unprocessed inputs again
    match repo.find_unprocessed().await {
        Ok(unprocessed) => {
            println!("  ⏳ Remaining unprocessed inputs: {}", unprocessed.len());
        }
        Err(e) => println!("  ❌ Error finding unprocessed inputs: {}", e),
    }

    // ============================================================================================
    // Example 6: Analytics
    // ============================================================================================
    println!("\n📊 Example 6: Analytics Summary");

    match repo.get_analytics_summary(None).await {
        Ok(summary) => {
            println!("  📈 Analytics Summary:");
            println!("    - Total inputs: {}", summary.total_inputs);
            println!("    - Processed inputs: {}", summary.processed_inputs);
            println!("    - Unique sessions: {}", summary.unique_sessions);
            println!("    - Date range: {} to {}", 
                summary.date_range.0.format("%Y-%m-%d"), 
                summary.date_range.1.format("%Y-%m-%d"));
        }
        Err(e) => println!("  ❌ Error getting analytics: {}", e),
    }

    // ============================================================================================
    // Example 7: Advanced Filtering with Pagination
    // ============================================================================================
    println!("\n🔍📄 Example 7: Advanced Filtering with Pagination");

    let advanced_filter = BehaviorInputFilter {
        user_id: Some(user_id.clone()),
        processed: Some(false),
        ..Default::default()
    };

    match repo.find_by_filter_paginated(advanced_filter, 1, 10).await {
        Ok(result) => {
            println!("  🎯 Advanced filter results:");
            println!("    - Found {} unprocessed inputs for user", result.total);
            println!("    - Showing page 1 of {}", result.total_pages);
            
            for input in &result.items {
                println!("      - {} ({:?}, processed: {})", 
                    input.id, input.input_type, input.processed);
            }
        }
        Err(e) => println!("  ❌ Error with advanced filtering: {}", e),
    }

    // ============================================================================================
    // Example 8: Database Health Check
    // ============================================================================================
    println!("\n🏥 Example 8: Database Health Check");

    match db_manager.health_check().await {
        jd_storage::HealthStatus::Healthy => {
            println!("  ✅ Database is healthy");
        }
        jd_storage::HealthStatus::Unhealthy(reason) => {
            println!("  ❌ Database is unhealthy: {}", reason);
        }
        jd_storage::HealthStatus::Uninitialized => {
            println!("  ⚠️  Database is not initialized");
        }
    }

    // Show pool statistics
    if let Some(stats) = db_manager.pool_stats() {
        println!("  📊 Connection Pool Stats:");
        println!("    - Active connections: {}", stats.size);
        println!("    - Idle connections: {}", stats.idle);
        println!("    - Max connections: {}", stats.max_connections);
    }

    // ============================================================================================
    // Example 9: Cleanup (Optional)
    // ============================================================================================
    println!("\n🧹 Example 9: Cleanup");

    // Optionally delete the created test data
    let cleanup = env::var("CLEANUP_TEST_DATA").unwrap_or_else(|_| "false".to_string());
    
    if cleanup.to_lowercase() == "true" {
        println!("  🗑️  Cleaning up test data...");
        
        let filter = BehaviorInputFilter {
            user_id: Some(user_id),
            ..Default::default()
        };
        
        match repo.delete_by_filter(filter).await {
            Ok(deleted_count) => {
                println!("  ✅ Deleted {} test behavior inputs", deleted_count);
            }
            Err(e) => {
                println!("  ❌ Error during cleanup: {}", e);
            }
        }
    } else {
        println!("  💾 Test data preserved. Set CLEANUP_TEST_DATA=true to clean up.");
    }

    // Close database connection
    db_manager.close().await;
    println!("\n🏁 ZK-Persona Database Example completed successfully!");

    Ok(())
}

/// Mask password in database URL for secure logging
fn mask_password(url: &str) -> String {
    if let Some(at_pos) = url.find('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            let mut masked = url.to_string();
            masked.replace_range((colon_pos + 1)..at_pos, "****");
            return masked;
        }
    }
    url.to_string()
}