#!/bin/bash

# Setup Ngrok for Local GitHub Webhook Testing
set -e

echo "🌐 Setting up Ngrok for Local GitHub Webhook Testing"
echo "================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Check if ngrok is installed
print_step "1. Checking ngrok installation"

if ! command -v ngrok &> /dev/null; then
    print_warning "ngrok is not installed. Installing..."
    
    # Detect OS
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS - Handle M1/ARM64 and Intel
        print_status "Detected macOS. Installing ngrok..."
        
        # Check if Homebrew is installed
        if command -v brew &> /dev/null; then
            # Try to uninstall any existing broken installation
            brew uninstall --cask ngrok 2>/dev/null || true
            brew untap ngrok/ngrok 2>/dev/null || true
            
            # Clean up any existing ngrok installations
            rm -rf /opt/homebrew/Caskroom/ngrok 2>/dev/null || true
            rm -rf /usr/local/Caskroom/ngrok 2>/dev/null || true
            
            # Install using the official tap
            print_status "Installing ngrok via Homebrew..."
            brew tap ngrok/ngrok || true
            brew install ngrok
            
            # If that fails, try installing via cask
            if ! command -v ngrok &> /dev/null; then
                print_warning "Homebrew formula failed, trying cask..."
                brew install --cask ngrok
            fi
        else
            # Direct download for M1 Macs without Homebrew
            print_status "Installing ngrok directly for Apple Silicon..."
            curl -s https://bin.equinox.io/c/bNyj1mQVY4c/ngrok-v3-stable-darwin-arm64.zip -o ngrok.zip
            unzip -o ngrok.zip
            sudo mv ngrok /usr/local/bin/ngrok
            rm ngrok.zip
            chmod +x /usr/local/bin/ngrok
        fi
        
        # Verify installation
        if ! command -v ngrok &> /dev/null; then
            print_error "Failed to install ngrok. Please install manually:"
            echo "1. Download from: https://ngrok.com/download"
            echo "2. Choose 'Mac (Apple Silicon)' for M1/M2 Macs"
            echo "3. Unzip and move to /usr/local/bin/"
            exit 1
        fi
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux
        curl -s https://ngrok-agent.s3.amazonaws.com/ngrok.asc | sudo tee /etc/apt/trusted.gpg.d/ngrok.asc >/dev/null
        echo "deb https://ngrok-agent.s3.amazonaws.com buster main" | sudo tee /etc/apt/sources.list.d/ngrok.list
        sudo apt update && sudo apt install ngrok
    else
        print_error "Unsupported OS. Please install ngrok manually from: https://ngrok.com/download"
        exit 1
    fi
fi

print_status "ngrok is installed ✓"

# Check if ngrok is authenticated
print_step "2. Checking ngrok authentication"

if ! ngrok config check &> /dev/null; then
    print_warning "ngrok is not authenticated"
    echo
    echo "Please sign up for a free ngrok account at: https://dashboard.ngrok.com/signup"
    echo "Then get your authtoken from: https://dashboard.ngrok.com/get-started/your-authtoken"
    echo
    read -p "Enter your ngrok authtoken: " NGROK_TOKEN
    
    if [ -n "$NGROK_TOKEN" ]; then
        ngrok config add-authtoken "$NGROK_TOKEN"
        print_status "ngrok authenticated successfully ✓"
    else
        print_error "No authtoken provided. Please run 'ngrok config add-authtoken YOUR_TOKEN' manually"
        exit 1
    fi
else
    print_status "ngrok is already authenticated ✓"
fi

# Start ngrok
print_step "3. Starting ngrok tunnel"

# Kill any existing ngrok processes
pkill -f ngrok || true

# Start ngrok in background
print_status "Starting ngrok on port 8080..."
ngrok http 8080 > /dev/null &
NGROK_PID=$!

# Wait for ngrok to start
sleep 3

# Get ngrok URL
print_step "4. Getting ngrok URL"

NGROK_URL=$(curl -s http://localhost:4040/api/tunnels | jq -r '.tunnels[0].public_url')

if [ -z "$NGROK_URL" ] || [ "$NGROK_URL" == "null" ]; then
    print_error "Failed to get ngrok URL. Make sure ngrok is running properly."
    print_warning "You can check ngrok status at: http://localhost:4040"
    exit 1
fi

print_status "Ngrok URL: $NGROK_URL"

# Generate webhook secret
print_step "5. Generating webhook secret"

WEBHOOK_SECRET=$(openssl rand -hex 32)
print_status "Generated webhook secret: $WEBHOOK_SECRET"

# Update .env file
print_step "6. Updating .env file"

# Backup existing .env if it exists
if [ -f .env ]; then
    cp .env .env.backup
    print_status "Backed up existing .env to .env.backup"
fi

# Create or update .env
cat > .env.ngrok << EOF
# Ngrok Configuration for Local Testing
WEBHOOK_BASE_URL=$NGROK_URL
GITHUB_WEBHOOK_SECRET=$WEBHOOK_SECRET

# Copy these values from your .env file if they exist
DATABASE_URL=postgres://jayden:postgres@localhost:5432/jaydenblog
REDIS_URL=redis://localhost:6379/
WEB_ADDR=0.0.0.0:8080
HOST=0.0.0.0
PORT=8080

# GitHub App Configuration (update with your values)
# GITHUB_APP_ID=your_app_id
# GITHUB_PRIVATE_KEY_PATH=./github-app-private-key.pem

# OR Personal Token
# GITHUB_TOKEN=ghp_your_personal_token

# Logging
RUST_LOG=info
RUST_BACKTRACE=1
EOF

print_status "Created .env.ngrok file"

# Update test script with correct values
print_step "7. Updating test script"

# Create a local version of the test script
cp test_github_integration_full.sh test_github_local.sh

# Update the webhook secret and base URL
sed -i.bak "s|WEBHOOK_SECRET=\"your_webhook_secret_here\"|WEBHOOK_SECRET=\"$WEBHOOK_SECRET\"|g" test_github_local.sh
sed -i.bak "s|BASE_URL=\"http://localhost:8080\"|BASE_URL=\"$NGROK_URL\"|g" test_github_local.sh

# Make it executable
chmod +x test_github_local.sh

print_status "Created test_github_local.sh with updated configuration"

# Create ngrok info file
print_step "8. Creating ngrok info file"

cat > ngrok_info.txt << EOF
Ngrok Tunnel Information
========================
Public URL: $NGROK_URL
Webhook URL: $NGROK_URL/api/v1/github/webhook
Webhook Secret: $WEBHOOK_SECRET
Ngrok PID: $NGROK_PID
Dashboard: http://localhost:4040

GitHub App Configuration:
1. Go to your GitHub App settings
2. Update Webhook URL to: $NGROK_URL/api/v1/github/webhook
3. Update Webhook Secret to: $WEBHOOK_SECRET
4. Save changes

To stop ngrok: kill $NGROK_PID
EOF

print_status "Created ngrok_info.txt with connection details"

# Display summary
echo
echo "================================================="
print_status "Ngrok setup complete! 🎉"
echo
echo "📋 Configuration Summary:"
echo "  - Public URL: $NGROK_URL"
echo "  - Webhook URL: $NGROK_URL/api/v1/github/webhook"
echo "  - Webhook Secret: $WEBHOOK_SECRET"
echo "  - Ngrok Dashboard: http://localhost:4040"
echo
echo "📝 Next Steps:"
echo "1. Start your application with ngrok environment:"
echo "   export $(cat .env.ngrok | xargs) && cargo run --bin web_server"
echo
echo "2. Update your GitHub App webhook settings:"
echo "   - Webhook URL: $NGROK_URL/api/v1/github/webhook"
echo "   - Webhook Secret: $WEBHOOK_SECRET"
echo
echo "3. Run tests with:"
echo "   ./test_github_local.sh"
echo
echo "4. Monitor webhook requests at:"
echo "   http://localhost:4040/inspect/http"
echo
echo "⚠️  Note: This ngrok URL will change when you restart ngrok!"
echo "================================================="

# Create helper script to stop ngrok
cat > stop_ngrok.sh << 'EOF'
#!/bin/bash
echo "Stopping ngrok..."
pkill -f ngrok || echo "ngrok was not running"
echo "ngrok stopped"
EOF
chmod +x stop_ngrok.sh

print_status "Created stop_ngrok.sh helper script"