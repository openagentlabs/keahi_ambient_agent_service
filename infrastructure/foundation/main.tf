terraform {
  required_version = ">= 1.0"
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 5.0"
    }
  }
}

# Configure the Google Cloud Provider
provider "google" {
  project = var.project_id
  
  # Authentication using service account key
  credentials = file("/home/keith/Downloads/keahi-ambient-agent-service-d9c5c0e3f93a.json")
}

# Enable required APIs
resource "google_project_service" "cloudresourcemanager_api" {
  project = var.project_id
  service = "cloudresourcemanager.googleapis.com"

  disable_dependent_services = true
  disable_on_destroy         = false
}

resource "google_project_service" "firestore_api" {
  project = var.project_id
  service = "firestore.googleapis.com"

  disable_dependent_services = true
  disable_on_destroy         = false

  depends_on = [google_project_service.cloudresourcemanager_api]
}

resource "google_project_service" "pubsub_api" {
  project = var.project_id
  service = "pubsub.googleapis.com"

  disable_dependent_services = true
  disable_on_destroy         = false

  depends_on = [google_project_service.cloudresourcemanager_api]
}

# Call the Firestore module
module "firestore" {
  source = "./modules/firestore"

  project_id = var.project_id
  database_name = var.database_name

  depends_on = [google_project_service.firestore_api]
}

# Create Pub/Sub topic for client events
resource "google_pubsub_topic" "client_events" {
  name    = var.pubsub_topic_name
  project = var.project_id

  depends_on = [google_project_service.pubsub_api]
}

resource "null_resource" "init_firestore_collection" {
  depends_on = [module.firestore]

  provisioner "local-exec" {
    command = "GOOGLE_APPLICATION_CREDENTIALS=/home/keith/Downloads/keahi-ambient-agent-service-d9c5c0e3f93a.json python3 create_registered_clients.py ${var.database_name}"
  }
}

# Output the Firestore database details
output "firestore_database_name" {
  description = "The name of the Firestore database"
  value       = module.firestore.database_name
}

output "firestore_database_id" {
  description = "The ID of the Firestore database"
  value       = module.firestore.database_id
}

output "firestore_database_location" {
  description = "The location of the Firestore database"
  value       = module.firestore.database_location
}

# Output the Pub/Sub topic details
output "pubsub_topic_name" {
  description = "The name of the Pub/Sub topic"
  value       = google_pubsub_topic.client_events.name
}

output "pubsub_topic_id" {
  description = "The ID of the Pub/Sub topic"
  value       = google_pubsub_topic.client_events.id
}

output "pubsub_topic_full_name" {
  description = "The full name of the Pub/Sub topic"
  value       = google_pubsub_topic.client_events.name
} 