#!/bin/bash

# Signal Manager Service Infrastructure Setup Script
# This script helps you set up GCP authentication and initialize Terraform

set -e

echo "🚀 Setting up Signal Manager Service Infrastructure..."

# Check if gcloud is installed
if ! command -v gcloud &> /dev/null; then
    echo "❌ Google Cloud SDK is not installed."
    echo "Please install it from: https://cloud.google.com/sdk/docs/install"
    exit 1
fi

# Check if terraform is installed
if ! command -v terraform &> /dev/null; then
    echo "❌ Terraform is not installed."
    echo "Please install it from: https://www.terraform.io/downloads"
    exit 1
fi

echo "✅ Prerequisites check passed"

# Check for service account key
SERVICE_ACCOUNT_KEY="/home/keith/Downloads/keahi-ambient-agent-service-d9c5c0e3f93a.json"
if [ -f "$SERVICE_ACCOUNT_KEY" ]; then
    echo "✅ Found service account key file at $SERVICE_ACCOUNT_KEY"
    echo "Using service account authentication"
else
    echo "🔐 Setting up GCP authentication..."
    echo "This will open a browser window for authentication."
    gcloud auth application-default login
fi

# Get the current project
CURRENT_PROJECT=$(gcloud config get-value project 2>/dev/null || echo "")

if [ -z "$CURRENT_PROJECT" ]; then
    echo "❌ No GCP project is set."
    echo "Please set a project using: gcloud config set project YOUR_PROJECT_ID"
    exit 1
fi

echo "✅ Using GCP project: $CURRENT_PROJECT"

# Create terraform.tfvars if it doesn't exist
if [ ! -f "terraform.tfvars" ]; then
    echo "📝 Creating terraform.tfvars with your project ID..."
    cat > terraform.tfvars << EOF
# GCP Project Configuration
project_id = "$CURRENT_PROJECT"

# Region (London - europe-west2)
region = "europe-west2"

# Database name
database_name = "signal-manager-service-db"
EOF
    echo "✅ Created terraform.tfvars"
else
    echo "ℹ️  terraform.tfvars already exists"
fi

# Initialize Terraform
echo "🔧 Initializing Terraform..."
terraform init

echo "✅ Setup complete!"
echo ""
echo "Next steps:"
echo "1. Review the configuration: terraform plan"
echo "2. Deploy the infrastructure: terraform apply"
echo ""
echo "To destroy the infrastructure: terraform destroy" 