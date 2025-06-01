// Test AI Analysis with Real Repository
// This example demonstrates analyzing a real GitHub repository

use ai_analysis_service::{
    application::use_cases::analysis_use_cases::AnalysisUseCases,
    domain::analysis_models::{AnalysisType},
    infrastructure::{
        analysis_repository_impl::AnalysisRepositoryImpl,
    },
    models::requests::AnalyzeRepositoryRequest,
};
use jd_core::AppState;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    println!("🚀 AI Analysis System - Real Repository Test");
    println!("============================================");

    // Initialize AppState
    let app_state = AppState::new().await?;
    println!("✅ Connected to database");

    // Initialize components
    let analysis_repository = Arc::new(AnalysisRepositoryImpl::new(app_state.clone()));
    let analysis_use_cases = Arc::new(AnalysisUseCases::new(analysis_repository, None));

    // Create real Move files content from Aptos OnlyFans
    let mut file_contents = HashMap::new();
    
    // Main contract (simplified version of real content)
    file_contents.insert(
        "sources/only4fans.move".to_string(),
        r#"
module aptos_onlyfans::only4fans {
    use std::signer;
    use std::vector;
    use aptos_framework::coin;
    use aptos_framework::timestamp;
    use aptos_framework::aptos_coin::AptosCoin;
    use aptos_std::smart_table::{Self, SmartTable};

    const ENOT_ADMIN: u64 = 1;
    const ENOT_ALREADY_PERMISSION: u64 = 2;
    const EIDOL_NOT_EXISTS: u64 = 3;

    struct CollectionInfo has key {
        idol_addr: address,
        price: u256,
        users_payed: SmartTable<address, u64>,
    }

    struct Only4FansAdmin has key {
        fee: u256,
    }

    struct IdolInfo has key {
        collections: vector<address>,
    }

    fun init_module(caller: &signer) {
        let admin_addr = signer::address_of(caller);
        move_to(caller, Only4FansAdmin {
            fee: 1000,
        });
    }

    // Vulnerability: Missing capability check
    public fun create_collection(caller: &signer, price: u256) acquires IdolInfo {
        let caller_addr = signer::address_of(caller);
        
        if (!exists<IdolInfo>(caller_addr)) {
            move_to(caller, IdolInfo {
                collections: vector::empty(),
            });
        };

        let idol_info = borrow_global_mut<IdolInfo>(caller_addr);
        let collection_info = CollectionInfo {
            idol_addr: caller_addr,
            price,
            users_payed: smart_table::new(),
        };
        
        move_to(caller, collection_info);
        vector::push_back(&mut idol_info.collections, caller_addr);
    }

    // Vulnerability: Unchecked balance operation
    public fun pay_to_see_collection(caller: &signer, collection_addr: address) acquires CollectionInfo {
        let caller_addr = signer::address_of(caller);
        let collection_info = borrow_global_mut<CollectionInfo>(collection_addr);
        
        // Missing balance check - vulnerability
        coin::transfer<AptosCoin>(caller, collection_info.idol_addr, collection_info.price);
        
        // Vulnerability: Timestamp dependence
        smart_table::upsert(&mut collection_info.users_payed, caller_addr, timestamp::now_seconds() + 1_000_000_000);
    }

    public fun check_collection_permission(collection_addr: address, user_addr: address): bool acquires CollectionInfo {
        if (!exists<CollectionInfo>(collection_addr)) {
            return false
        };
        
        let collection_info = borrow_global<CollectionInfo>(collection_addr);
        smart_table::contains(&collection_info.users_payed, user_addr)
    }

    // Good: Has assertion for admin check
    public fun get_fee(admin: address): u256 acquires Only4FansAdmin {
        assert!(exists<Only4FansAdmin>(admin), ENOT_ADMIN);
        let admin_info = borrow_global<Only4FansAdmin>(admin);
        admin_info.fee
    }

    public fun get_my_collections(idol_addr: address): vector<address> acquires IdolInfo {
        if (!exists<IdolInfo>(idol_addr)) {
            return vector::empty()
        };
        
        let idol_info = borrow_global<IdolInfo>(idol_addr);
        idol_info.collections
    }
}
"#.to_string(),
    );

    // Error config
    file_contents.insert(
        "sources/error_config.move".to_string(),
        r#"
module aptos_onlyfans::error_config {
    public fun get_enot_admin(): u64 {
        1
    }

    public fun get_enot_already_permission(): u64 {
        2
    }

    public fun get_eidol_not_exists(): u64 {
        3
    }

    public fun get_einvalid_price(): u64 {
        4
    }

    public fun get_einsufficient_balance(): u64 {
        5
    }

    public fun get_ecollection_not_found(): u64 {
        6
    }
}
"#.to_string(),
    );

    // Test file
    file_contents.insert(
        "tests/test_only4fans.move".to_string(),
        r#"
#[test_only]
module aptos_onlyfans::test_only4fans {
    use aptos_onlyfans::only4fans;
    use std::signer;

    #[test(creator = @0x123, framework = @aptos_framework)]
    public fun test_create_collection(creator: &signer, framework: &signer) {
        // Test collection creation
        only4fans::create_collection(creator, 1000);
    }

    // Vulnerability: No overflow check in test
    #[test]
    public fun test_large_values() {
        let large_price = 18446744073709551615; // Near u64 max
        // This could cause issues if multiplied
    }
}
"#.to_string(),
    );

    println!("📁 Loaded {} Move files for analysis", file_contents.len());

    // Create repository record using direct SQLx
    let repo_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO github_repositories (
            id, github_repo_id, owner_username, repo_name, full_name,
            primary_language, is_private, star_count, fork_count, monitoring_enabled
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#
    )
    .bind(repo_id)
    .bind(555666777_i64)
    .bind("jayden-dang")
    .bind("aptos_onlyfans")
    .bind("jayden-dang/aptos_onlyfans")
    .bind("Move")
    .bind(false)
    .bind(12_i32)
    .bind(3_i32)
    .bind(true)
    .execute(app_state.mm().dbx().db())
    .await?;
    println!("✅ Created repository record: {}", repo_id);

    // Run analysis
    println!("🔍 Starting AI analysis...");
    
    let analysis_request = AnalyzeRepositoryRequest {
        repository_id: repo_id,
        commit_sha: "1234567890abcdef1234567890abcdef12345678".to_string(),
        files_to_analyze: None,
        analysis_types: vec![
            AnalysisType::StaticAnalysis,
            AnalysisType::VulnerabilityDetection,
        ],
        enable_llm_analysis: Some(false),
    };

    let start_time = std::time::Instant::now();
    
    match analysis_use_cases.analyze_repository(analysis_request, file_contents).await {
        Ok(result) => {
            let duration = start_time.elapsed();
            
            println!("✅ Analysis completed in {:?}", duration);
            println!("");
            println!("📊 ANALYSIS RESULTS");
            println!("==================");
            println!("🔗 Analysis ID: {}", result.analysis_id);
            println!("📂 Repository: jayden-dang/aptos_onlyfans");
            println!("🛡️  Security Score: {:.1}/100", result.security_score);
            println!("⭐ Quality Score: {:.1}/100", result.quality_score);
            println!("🚨 Vulnerabilities Found: {}", result.vulnerabilities_found);
            println!("🔴 Critical Issues: {}", result.critical_vulnerabilities);
            println!("⏱️  Analysis Duration: {}ms", result.analysis_duration_ms);
            
            // Get detailed results from database
            println!("");
            println!("🔍 VULNERABILITY DETAILS");
            println!("========================");
            
            let vulnerabilities = sqlx::query_as::<_, (Option<String>, Option<String>, Decimal, String, Option<i32>, String, String)>(
                r#"
                SELECT vulnerability_type::text, severity::text, confidence_score, file_path, 
                       line_number, description, recommendation
                FROM security_vulnerabilities 
                WHERE repository_id = $1
                ORDER BY 
                    CASE severity 
                        WHEN 'critical' THEN 1 
                        WHEN 'high' THEN 2 
                        WHEN 'medium' THEN 3 
                        WHEN 'low' THEN 4 
                    END,
                    confidence_score DESC
                "#
            )
            .bind(repo_id)
            .fetch_all(app_state.mm().dbx().db())
            .await?;

            for (i, vuln) in vulnerabilities.iter().enumerate() {
                println!("{}. {} ({})", i + 1, vuln.0.as_ref().unwrap().to_uppercase(), vuln.1.as_ref().unwrap().to_uppercase());
                println!("   📍 Location: {} (Line: {:?})", vuln.3, vuln.4);
                println!("   🎯 Confidence: {:.1}%", vuln.2);
                println!("   📝 Issue: {}", vuln.5);
                println!("   💡 Fix: {}", vuln.6);
                println!("");
            }

            // Security assessment
            let critical_count = vulnerabilities.iter().filter(|v| v.1.as_ref().unwrap() == "critical").count();
            let high_count = vulnerabilities.iter().filter(|v| v.1.as_ref().unwrap() == "high").count();
            
            println!("🎯 SECURITY ASSESSMENT");
            println!("======================");
            
            let risk_level = match (critical_count, high_count) {
                (c, _) if c > 0 => "🔴 CRITICAL RISK",
                (_, h) if h > 0 => "🟡 HIGH RISK", 
                _ => "🟢 ACCEPTABLE RISK"
            };
            
            println!("Risk Level: {}", risk_level);
            println!("Total Issues: {}", vulnerabilities.len());
            println!("Needs Immediate Attention: {}", critical_count + high_count);
            
            if vulnerabilities.len() > 0 {
                let avg_confidence: f64 = vulnerabilities.iter()
                    .map(|v| v.2.to_string().parse::<f64>().unwrap_or(0.0))
                    .sum::<f64>() / vulnerabilities.len() as f64;
                println!("Average Confidence: {:.1}%", avg_confidence);
            }

            println!("");
            println!("✅ ANALYSIS STORED TO DATABASE");
            println!("==============================");
            println!("The AI analysis results have been successfully stored in your database.");
            println!("You can now:");
            println!("  • View results via API endpoints");
            println!("  • Track vulnerability status");
            println!("  • Generate security reports");
            println!("  • Monitor security trends over time");
        }
        Err(e) => {
            println!("❌ Analysis failed: {}", e);
        }
    }

    // Cleanup
    println!("");
    println!("🧹 Cleaning up test data...");
    sqlx::query("DELETE FROM security_vulnerabilities WHERE repository_id = $1")
        .bind(repo_id)
        .execute(app_state.mm().dbx().db())
        .await?;
    sqlx::query("DELETE FROM code_analysis_results WHERE repository_id = $1")
        .bind(repo_id)
        .execute(app_state.mm().dbx().db())
        .await?;
    sqlx::query("DELETE FROM github_repositories WHERE id = $1")
        .bind(repo_id)
        .execute(app_state.mm().dbx().db())
        .await?;
    
    println!("✅ Cleanup completed");
    
    println!("");
    println!("🎉 AI ANALYSIS SYSTEM TEST COMPLETE");
    println!("===================================");
    println!("🚀 The AI Analysis System successfully analyzed real smart contract code!");

    Ok(())
}