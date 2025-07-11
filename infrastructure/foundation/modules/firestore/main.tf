# Create Firestore database in London region (europe-west2)
resource "google_firestore_database" "database" {
  name        = var.database_name
  location_id = "europe-west2" # London region
  type        = "FIRESTORE_NATIVE"
  project     = var.project_id
} 