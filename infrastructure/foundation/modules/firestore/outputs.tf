output "database_name" {
  description = "The name of the Firestore database"
  value       = google_firestore_database.database.name
}

output "database_id" {
  description = "The ID of the Firestore database"
  value       = google_firestore_database.database.id
}

output "database_location" {
  description = "The location of the Firestore database"
  value       = google_firestore_database.database.location_id
} 