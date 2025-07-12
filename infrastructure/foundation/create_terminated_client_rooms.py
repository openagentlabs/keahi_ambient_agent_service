import datetime
import os
import sys
import logging
from google.cloud import firestore
from google.oauth2 import service_account

# Set up logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

def drop_collection_if_exists(db, collection_name):
    """Drop a collection if it exists"""
    try:
        collection_ref = db.collection(collection_name)
        docs = collection_ref.stream()
        doc_count = 0
        
        # Count documents and delete them
        for doc in docs:
            doc.reference.delete()
            doc_count += 1
        
        if doc_count > 0:
            logger.info(f"Dropped {doc_count} documents from existing '{collection_name}' collection")
        else:
            logger.info(f"Collection '{collection_name}' was empty or didn't exist")
            
    except Exception as e:
        logger.warning(f"Error while dropping collection '{collection_name}': {e}")

def main():
    try:
        # Get database name from command-line argument or use default
        if len(sys.argv) > 1:
            database_name = sys.argv[1]
        else:
            database_name = "signal-manager-service-db"
        
        logger.info(f"Using database: {database_name}")

        # Path to your service account key
        SERVICE_ACCOUNT_PATH = os.environ.get("GOOGLE_APPLICATION_CREDENTIALS", "/home/keith/Downloads/keahi-ambient-agent-service-d9c5c0e3f93a.json")
        
        if not os.path.exists(SERVICE_ACCOUNT_PATH):
            logger.error(f"Service account file not found: {SERVICE_ACCOUNT_PATH}")
            sys.exit(1)

        logger.info(f"Using service account: {SERVICE_ACCOUNT_PATH}")
        
        credentials = service_account.Credentials.from_service_account_file(SERVICE_ACCOUNT_PATH)
        db = firestore.Client(credentials=credentials, project=credentials.project_id, database=database_name)
        
        logger.info(f"Connected to Firestore database: {database_name}")

        # Drop existing collection if it exists
        collection_name = "client_rooms_terminated"
        logger.info(f"Checking if '{collection_name}' collection exists...")
        drop_collection_if_exists(db, collection_name)

        # Sample document for terminated rooms collection
        terminated_room_doc = {
            "id": "terminated_room_example_id",
            "room_id": "room_terminated_123",
            "terminated_at": firestore.SERVER_TIMESTAMP,
            "termination_recorded_at": firestore.SERVER_TIMESTAMP,
            "room_data": {
                "room_name": "Sample Room",
                "participants": ["user1", "user2"],
                "settings": {"max_participants": 10}
            },
            "termination_reason": "Normal closure",
            "terminated_by": "client_terminator_1",
            "metadata": {
                "note": "sample termination",
                "session_duration": "2h 30m"
            },
            "record_created_at": firestore.SERVER_TIMESTAMP
        }

        # Add the document to the collection
        collection_ref = db.collection(collection_name)
        logger.info(f"Creating '{collection_name}' collection...")

        doc_ref = collection_ref.document(terminated_room_doc["id"])
        doc_ref.set(terminated_room_doc)
        logger.info(f"Created terminated room document: {terminated_room_doc['id']}")

        print(f"Created '{collection_name}' collection in database '{database_name}' with sample document:")
        print(f"Document: {terminated_room_doc}")
        logger.info(f"Successfully created '{collection_name}' collection with sample document")
        
    except Exception as e:
        logger.error(f"Error creating '{collection_name}' collection: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main() 