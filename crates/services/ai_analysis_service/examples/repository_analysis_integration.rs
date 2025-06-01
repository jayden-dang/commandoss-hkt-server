use ai_analysis_service::{
    application::use_cases::analysis_use_cases::AnalysisUseCases,
    domain::analysis_models::AnalysisType,
    infrastructure::{
        analysis_repository_impl::AnalysisRepositoryImpl,
        github_integration::{GitHubIntegrationService, MockGitHubFileProvider},
        llm_client::LLMClient,
    },
    models::requests::AnalyzeRepositoryRequest,
};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// Example of how to integrate AI analysis when adding a new repository
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::init();

    // This would normally come from your database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:password@localhost/database".to_string());
    
    let pool = PgPool::connect(&database_url).await?;

    // Initialize AI analysis components
    let analysis_repository = Arc::new(AnalysisRepositoryImpl::new(pool));
    
    // Optional: Initialize LLM client if OpenAI API key is available
    let llm_provider = std::env::var("OPENAI_API_KEY")
        .ok()
        .map(|api_key| Arc::new(LLMClient::new_openai(api_key)) as Arc<dyn ai_analysis_service::domain::llm_provider_trait::LLMProvider>);
    
    let analysis_use_cases = Arc::new(AnalysisUseCases::new(
        analysis_repository.clone(),
        llm_provider,
    ));

    // Initialize GitHub integration service
    let github_client = Arc::new(MockGitHubFileProvider);
    let github_integration = GitHubIntegrationService::new(analysis_use_cases.clone())
        .with_github_client(github_client);

    // Example: Auto-analyze a newly added repository
    let repository_id = Uuid::new_v4();
    let owner = "sui-foundation";
    let repo_name = "sui-example-contracts";

    println!("Starting analysis for repository: {}/{}", owner, repo_name);

    match github_integration
        .auto_analyze_repository(repository_id, owner, repo_name, None)
        .await
    {
        Ok(analysis_result) => {
            println!("✅ Analysis completed successfully!");
            println!("📊 Security Score: {:.1}/100", analysis_result.security_score);
            println!("📈 Quality Score: {:.1}/100", analysis_result.quality_score);
            println!("🔍 Vulnerabilities Found: {}", analysis_result.vulnerabilities_found);
            println!("🚨 Critical Vulnerabilities: {}", analysis_result.critical_vulnerabilities);
            println!("⏱️  Analysis Duration: {}ms", analysis_result.analysis_duration_ms);
            
            // Get detailed analysis
            let detailed_analysis = analysis_use_cases
                .get_detailed_analysis(analysis_result.analysis_id)
                .await?;
            
            println!("\n📝 Detailed Analysis:");
            for vulnerability in &detailed_analysis.analysis_result.vulnerabilities {
                println!("  - {} ({}): {}", 
                    vulnerability.vulnerability_type, 
                    vulnerability.severity, 
                    vulnerability.description
                );
                if let Some(line) = vulnerability.line_number {
                    println!("    📍 File: {} (Line: {})", vulnerability.file_path, line);
                }
                println!("    💡 Recommendation: {}", vulnerability.recommendation);
                println!();
            }
            
            // Example: Simulate webhook analysis (e.g., on push event)
            println!("🔄 Simulating webhook analysis...");
            let changed_files = vec!["sources/example.move".to_string()];
            
            if let Some(webhook_result) = github_integration
                .analyze_on_webhook(
                    repository_id,
                    owner,
                    repo_name,
                    "abc123",
                    changed_files,
                )
                .await?
            {
                println!("✅ Webhook analysis completed!");
                println!("📊 New Security Score: {:.1}/100", webhook_result.security_score);
                println!("🔍 New Vulnerabilities Found: {}", webhook_result.vulnerabilities_found);
            }
            
            // Get analysis status
            let status = github_integration
                .get_repository_analysis_status(repository_id)
                .await?;
            
            println!("\n📈 Repository Analysis Status:");
            println!("  Total Analyses: {}", status.total_analyses);
            println!("  Total Vulnerabilities: {}", status.vulnerability_statistics.total_vulnerabilities);
            println!("  Critical: {}", status.vulnerability_statistics.critical_count);
            println!("  High: {}", status.vulnerability_statistics.high_count);
            println!("  Medium: {}", status.vulnerability_statistics.medium_count);
            println!("  Low: {}", status.vulnerability_statistics.low_count);
            println!("  False Positives: {}", status.vulnerability_statistics.false_positive_count);
            println!("  Fixed: {}", status.vulnerability_statistics.fixed_count);
        }
        Err(e) => {
            println!("❌ Analysis failed: {}", e);
        }
    }

    Ok(())
}

/// Example of direct code analysis (useful for IDE integrations)
#[allow(dead_code)]
async fn example_direct_code_analysis() -> Result<(), Box<dyn std::error::Error>> {
    let analysis_repository = Arc::new(AnalysisRepositoryImpl::new(
        PgPool::connect("postgresql://localhost/test").await?
    ));
    
    let analysis_use_cases = Arc::new(AnalysisUseCases::new(analysis_repository, None));

    let code_request = ai_analysis_service::models::requests::AnalyzeCodeRequest {
        code: r#"
module test::example {
    public fun unsafe_transfer(item: Item, recipient: address) {
        // Missing capability check - this is a vulnerability
        transfer::public_transfer(item, recipient);
    }
}
"#.to_string(),
        file_path: "test.move".to_string(),
        analysis_types: vec![AnalysisType::StaticAnalysis],
    };

    let result = analysis_use_cases.analyze_code(code_request).await?;
    
    println!("Code Analysis Result:");
    println!("Security Score: {:.1}", result.security_score);
    println!("Quality Score: {:.1}", result.quality_score);
    println!("Vulnerabilities: {}", result.vulnerabilities.len());
    
    for vuln in result.vulnerabilities {
        println!("  - {}: {}", vuln.vulnerability_type, vuln.description);
    }

    Ok(())
}