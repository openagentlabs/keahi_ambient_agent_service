# Signal Manager Service Infrastructure

This directory contains the Terraform configuration for deploying the Signal Manager Service infrastructure on Google Cloud Platform.

## Overview

The infrastructure includes:
- GCP project configuration
- Firestore API enablement
- Firestore database in London region (europe-west2)

## Prerequisites

1. **Google Cloud SDK**: Install and configure the Google Cloud SDK
2. **Terraform**: Install Terraform (version >= 1.0)
3. **GCP Project**: Have a GCP project ready
4. **Authentication**: Either:
   - Use Application Default Credentials: `gcloud auth application-default login`
   - Or use a Service Account Key file: `/home/keith/Downloads/keahi-ambient-agent-service-d9c5c0e3f93a.json`

## Quick Start

### Option 1: Automated Setup (Recommended)

1. **Run the setup script**:
   ```bash
   ./setup.sh
   ```
   This will:
   - Check prerequisites (gcloud, terraform)
   - Authenticate with GCP
   - Create terraform.tfvars with your project ID
   - Initialize Terraform

2. **Deploy the infrastructure**:
   ```bash
   terraform plan
   terraform apply
   ```

### Option 2: Manual Setup

#### Using Application Default Credentials:
1. **Authenticate with GCP**:
   ```bash
   gcloud auth application-default login
   ```

2. **Configure variables**:
   ```bash
   cp terraform.tfvars.example terraform.tfvars
   # Edit terraform.tfvars with your GCP project ID
   ```

3. **Initialize and deploy**:
   ```bash
   terraform init
   terraform plan
   terraform apply
   ```

#### Using Service Account Key:
1. **Ensure service account key file exists**:
   ```bash
   # The service account key should be at:
   # /home/keith/Downloads/keahi-ambient-agent-service-d9c5c0e3f93a.json
   ```

2. **Configure variables**:
   ```bash
   cp terraform.tfvars.example terraform.tfvars
   # Edit terraform.tfvars with your GCP project ID
   ```

3. **Initialize and deploy**:
   ```bash
   terraform init
   terraform plan
   terraform apply
   ```

## Configuration

### Variables

- `project_id`: Your GCP project ID (required)
- `region`: GCP region for resources (defaults to `europe-west2` - London)
- `database_name`: Name of the Firestore database (defaults to `signal-manager-service-db`)

### Firestore Database

The Firestore database is configured with:
- **Location**: London (europe-west2)
- **Type**: Native Firestore
- **Name**: `signal-manager-service-db` (configurable)

## Outputs

After successful deployment, Terraform will output:
- `firestore_database_name`: The name of the created database
- `firestore_database_id`: The ID of the database
- `firestore_database_location`: The location of the database

## Cleanup

To destroy the infrastructure:
```bash
terraform destroy
```

## Module Structure

```
infrastructure/signal_manager_service/
├── main.tf                 # Main orchestrator
├── variables.tf            # Variable definitions
├── terraform.tfvars.example # Example configuration
├── setup.sh               # Automated setup script
├── README.md              # This file
├── .gitignore             # Terraform gitignore
└── modules/
    └── firestore/
        ├── main.tf        # Firestore database resource
        ├── variables.tf   # Module variables
        └── outputs.tf     # Module outputs
``` 