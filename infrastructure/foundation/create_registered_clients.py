import datetime
import os
import sys
from google.cloud import firestore
from google.oauth2 import service_account

# Get database name from command-line argument or use default
if len(sys.argv) > 1:
    database_name = sys.argv[1]
else:
    database_name = "signal-manager-service-db"

# Path to your service account key
SERVICE_ACCOUNT_PATH = os.environ.get("GOOGLE_APPLICATION_CREDENTIALS", "/home/keith/Downloads/keahi-ambient-agent-service-d9c5c0e3f93a.json")

credentials = service_account.Credentials.from_service_account_file(SERVICE_ACCOUNT_PATH)
db = firestore.Client(credentials=credentials, project=credentials.project_id, database=database_name)

# Sample document data
sample_doc = {
    "id": "example_id",
    "client_id": "example_client_id",
    "registered_time": firestore.SERVER_TIMESTAMP,
    "disconnected": False,
    "disconnected_time": None
}

# Add the document to the collection
collection_ref = db.collection("registered_clients")
doc_ref = collection_ref.document(sample_doc["id"])
doc_ref.set(sample_doc)

print(f"Created 'registered_clients' collection in database '{database_name}' with sample document: {sample_doc}") 