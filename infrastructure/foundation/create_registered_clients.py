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
        collection_name = "registered_clients"
        logger.info(f"Checking if '{collection_name}' collection exists...")
        drop_collection_if_exists(db, collection_name)

        # Sample document data with room_id field
        sample_doc = {
            "id": "example_id",
            "client_id": "example_client_id",
            "auth_token": "example_auth_token",
            "room_id": "room_123",  # Associated room ID
            "capabilities": ["websocket", "video"],
            "registered_at": firestore.SERVER_TIMESTAMP,
            "last_seen": None,
            "status": "Active",
            "metadata": {
                "version": "1.0",
                "platform": "web"
            },
            "disconnected": False,
            "disconnected_time": None,
            "record_created_at": firestore.SERVER_TIMESTAMP
        }

        # Sample document without room association
        sample_doc_no_room = {
            "id": "example_id_no_room",
            "client_id": "example_client_id_no_room",
            "auth_token": "example_auth_token_no_room",
            "room_id": None,  # No room association
            "capabilities": ["websocket"],
            "registered_at": firestore.SERVER_TIMESTAMP,
            "last_seen": None,
            "status": "Active",
            "metadata": {
                "version": "1.0",
                "platform": "mobile"
            },
            "disconnected": False,
            "disconnected_time": None,
            "record_created_at": firestore.SERVER_TIMESTAMP
        }

        # Add the documents to the collection
        collection_ref = db.collection(collection_name)
        logger.info(f"Creating '{collection_name}' collection...")

        # Add document with room association
        doc_ref = collection_ref.document(sample_doc["id"])
        doc_ref.set(sample_doc)
        logger.info(f"Created document with room association: {sample_doc['id']}")

        # Add document without room association
        doc_ref_no_room = collection_ref.document(sample_doc_no_room["id"])
        doc_ref_no_room.set(sample_doc_no_room)
        logger.info(f"Created document without room association: {sample_doc_no_room['id']}")

        print(f"Created '{collection_name}' collection in database '{database_name}' with sample documents:")
        print(f"1. Document with room association: {sample_doc}")
        print(f"2. Document without room association: {sample_doc_no_room}")
        logger.info(f"Successfully created '{collection_name}' collection with sample documents")
        
    except Exception as e:
        logger.error(f"Error creating '{collection_name}' collection: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main() 