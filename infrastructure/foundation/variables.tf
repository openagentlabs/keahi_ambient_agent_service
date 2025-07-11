variable "project_id" {
  description = "The GCP project ID"
  type        = string
}

variable "region" {
  description = "The GCP region for resources"
  type        = string
  default     = "europe-west2" # London region
}

variable "database_name" {
  description = "The name of the Firestore database"
  type        = string
  default     = "signal-manager-service-db"
}

variable "pubsub_topic_name" {
  description = "The name of the Pub/Sub topic for client events"
  type        = string
  default     = "client_events"
}

variable "pubsub_topic_region" {
  description = "The region for the Pub/Sub topic"
  type        = string
  default     = "europe-west2"
} 