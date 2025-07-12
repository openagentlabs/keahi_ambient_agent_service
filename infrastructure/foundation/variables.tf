variable "project_id" {
  description = "The GCP project ID"
  type        = string
}

variable "database_name" {
  description = "The name of the Firestore database"
  type        = string
  default     = "signal-manager-service-db"
} 