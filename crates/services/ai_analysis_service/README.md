# AI Analysis Service

A comprehensive AI-powered analysis system for Sui Move smart contracts that provides static analysis, LLM-powered code review, vulnerability detection, and security scoring.

## Features

### 🔍 Static Analysis
- **Sui Move Pattern Detection**: Specialized vulnerability patterns for Sui Move smart contracts
- **Security Vulnerability Detection**: Access control, integer overflow, logic errors, timestamp dependencies
- **Code Quality Assessment**: Documentation, structure, and best practices analysis
- **Modular Design**: Easy to extend with additional languages (Ink!, Move, etc.)

### 🤖 LLM Integration
- **Multiple Provider Support**: OpenAI, Anthropic, and local models
- **Advanced Code Review**: Deep vulnerability analysis using large language models
- **Security Recommendations**: AI-generated fix suggestions with code examples
- **Context-Aware Analysis**: Understands Sui Move semantics and security patterns

### 📊 Vulnerability Database & Scoring
- **Comprehensive Scoring**: 0-100 security and quality scores
- **CVE Integration**: Links to known vulnerabilities
- **False Positive Filtering**: Machine learning-based confidence scoring
- **Vulnerability Tracking**: Status management (open, fixed, false positive)

### ⚡ Auto-Analysis Workflow
- **Repository Integration**: Automatic analysis when repositories are added
- **Webhook Support**: Analysis on push events and pull requests
- **Real-time Scoring**: Live security score updates
- **Batch Processing**: Efficient analysis of multiple files

## Architecture

```
ai_analysis_service/
├── domain/              # Core business logic
│   ├── analysis_engine.rs       # Main analysis orchestrator
│   ├── analysis_models.rs       # Domain models
│   ├── vulnerability_patterns.rs # Security pattern definitions
│   └── llm_provider_trait.rs    # LLM abstraction
├── infrastructure/     # External integrations
│   ├── static_analyzer.rs       # Sui Move static analysis
│   ├── llm_client.rs           # LLM API clients
│   ├── analysis_repository_impl.rs # Database operations
│   └── github_integration.rs   # GitHub integration
├── application/        # Use cases and handlers
│   ├── use_cases/
│   └── handlers/
└── models/            # Request/response models
```

## Usage

### Basic Repository Analysis

```rust
use ai_analysis_service::*;

// Setup
let analysis_repository = Arc::new(AnalysisRepositoryImpl::new(db_pool));
let llm_provider = Arc::new(LLMClient::new_openai(api_key));
let analysis_use_cases = Arc::new(AnalysisUseCases::new(
    analysis_repository,
    Some(llm_provider)
));

// Analyze repository
let request = AnalyzeRepositoryRequest {
    repository_id: repo_id,
    commit_sha: "main".to_string(),
    files_to_analyze: None, // Analyze all Move files
    analysis_types: vec![
        AnalysisType::StaticAnalysis,
        AnalysisType::LLMReview,
    ],
    enable_llm_analysis: Some(true),
};

let result = analysis_use_cases
    .analyze_repository(request, file_contents)
    .await?;

println!("Security Score: {:.1}/100", result.security_score);
println!("Vulnerabilities: {}", result.vulnerabilities_found);
```

### Auto-Analysis Integration

```rust
// Setup GitHub integration
let github_integration = GitHubIntegrationService::new(analysis_use_cases)
    .with_github_client(github_client);

// Auto-analyze new repository
let analysis_result = github_integration
    .auto_analyze_repository(repo_id, "owner", "repo", None)
    .await?;

// Webhook analysis on code changes
let webhook_result = github_integration
    .analyze_on_webhook(repo_id, "owner", "repo", "commit_sha", changed_files)
    .await?;
```

### Direct Code Analysis

```rust
let code_request = AnalyzeCodeRequest {
    code: move_code_string,
    file_path: "example.move".to_string(),
    analysis_types: vec![AnalysisType::StaticAnalysis],
};

let result = analysis_use_cases.analyze_code(code_request).await?;
```

## API Endpoints

### Repository Analysis
- `POST /api/v1/repositories/analyze` - Analyze repository
- `GET /api/v1/repositories/{id}/analysis/status` - Get analysis status
- `GET /api/v1/repositories/{id}/vulnerabilities` - List vulnerabilities
- `GET /api/v1/repositories/{id}/analysis/history` - Analysis history

### Analysis Management
- `GET /api/v1/analysis/{id}` - Get detailed analysis
- `POST /api/v1/code/analyze` - Analyze code snippet
- `PUT /api/v1/vulnerabilities/mark` - Mark vulnerability status

## Vulnerability Patterns

The system includes built-in patterns for common Sui Move vulnerabilities:

### Access Control Issues
- **Unauthorized Transfer**: Direct transfers without capability checks
- **Missing Capability Verification**: Functions missing capability parameters
- **Friend Module Abuse**: Inappropriate friend declarations

### Resource Management
- **Unchecked Balance Operations**: Balance operations without validation
- **Resource Leaks**: Improper object handling
- **Unauthorized Resource Access**: Missing ownership checks

### Logic Errors
- **Integer Overflow**: Arithmetic without bounds checking
- **Timestamp Dependencies**: Critical logic relying on timestamps
- **Input Validation**: Missing parameter validation

### Code Quality
- **Missing Documentation**: Undocumented public functions
- **Test Coverage**: Missing or inadequate tests
- **Error Handling**: Insufficient error handling

## Configuration

### Environment Variables
```bash
# Database
DATABASE_URL=postgresql://user:pass@localhost/db

# LLM Providers (optional)
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...

# Analysis Settings
ENABLE_LLM_ANALYSIS=true
MAX_FILE_SIZE_KB=10
ANALYSIS_TIMEOUT_MS=60000
```

### LLM Provider Setup

```rust
// OpenAI
let llm_client = LLMClient::new_openai(api_key);

// Anthropic
let llm_client = LLMClient::new_anthropic(api_key);

// Local model
let llm_client = LLMClient::new_local(
    "http://localhost:8080".to_string(),
    "local-model".to_string()
);
```

## Database Schema

The service uses the existing database tables:
- `code_analysis_results` - Analysis metadata and scores
- `security_vulnerabilities` - Vulnerability findings
- `github_repositories` - Repository information

## Development

### Adding New Vulnerability Patterns

```rust
let pattern = VulnerabilityPattern {
    id: "sui_new_vulnerability".to_string(),
    name: "New Vulnerability Type".to_string(),
    description: "Description of the vulnerability".to_string(),
    vulnerability_type: VulnerabilityType::Other("NewType".to_string()),
    severity: Severity::High,
    confidence_base: 80.0,
    pattern: PatternRule::Regex(r"pattern_regex".to_string()),
    recommendation: "How to fix this vulnerability".to_string(),
};

vulnerability_patterns.add_custom_pattern(pattern);
```

### Extending LLM Providers

```rust
#[async_trait]
impl LLMProvider for CustomLLMClient {
    async fn analyze_code(&self, request: LLMRequest) -> Result<LLMResponse> {
        // Custom implementation
    }
    
    async fn detect_vulnerabilities(&self, code: &str, file_path: &str) -> Result<CodeAnalysisResponse> {
        // Custom implementation
    }
    
    // ... other methods
}
```

## Performance Considerations

- **File Size Limits**: Large files are skipped in LLM analysis to avoid token limits
- **Concurrent Analysis**: Multiple files analyzed in parallel
- **Caching**: Analysis results cached in database
- **Rate Limiting**: Built-in rate limiting for LLM API calls
- **Incremental Analysis**: Only analyze changed files on webhooks

## Security

- **API Key Management**: Secure storage of LLM provider API keys
- **Input Validation**: All inputs validated before processing
- **Error Handling**: Graceful degradation when LLM services unavailable
- **Audit Logging**: All analysis activities logged for audit trails

## Monitoring

The service provides comprehensive metrics:
- Analysis success/failure rates
- Vulnerability detection accuracy
- LLM API usage and costs
- Processing time per analysis
- Security score trends over time