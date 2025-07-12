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
        collection_name = "client_in_terminated_room"
        logger.info(f"Checking if '{collection_name}' collection exists...")
        drop_collection_if_exists(db, collection_name)

        # Sample documents for clients in terminated room collection
        client_in_terminated_room_docs = [
            {
                "id": "client_terminated_1",
                "client_id": "client_001",
                "room_id": "room_terminated_123",
                "joined_at": firestore.SERVER_TIMESTAMP,
                "left_at": firestore.SERVER_TIMESTAMP,
                "termination_reason": "Room terminated by admin",
                "terminated_by": "admin_user",
                "final_status": "disconnected",
                "capabilities": ["websocket", "video", "audio"],
                "metadata": {
                    "user_agent": "Mozilla/5.0",
                    "ip_address": "192.168.1.100",
                    "session_duration": "45m 30s",
                    "last_activity": "2025-07-11T22:15:00Z"
                },
                "record_created_at": firestore.SERVER_TIMESTAMP
            },
            {
                "id": "client_terminated_2",
                "client_id": "client_002",
                "room_id": "room_terminated_123",
                "joined_at": firestore.SERVER_TIMESTAMP,
                "left_at": firestore.SERVER_TIMESTAMP,
                "termination_reason": "Room terminated by admin",
                "terminated_by": "admin_user",
                "final_status": "disconnected",
                "capabilities": ["websocket", "audio"],
                "metadata": {
                    "user_agent": "Mobile App v2.1",
                    "ip_address": "192.168.1.101",
                    "session_duration": "32m 15s",
                    "last_activity": "2025-07-11T22:12:30Z"
                },
                "record_created_at": firestore.SERVER_TIMESTAMP
            },
            {
                "id": "client_terminated_3",
                "client_id": "client_003",
                "room_id": "room_terminated_456",
                "joined_at": firestore.SERVER_TIMESTAMP,
                "left_at": firestore.SERVER_TIMESTAMP,
                "termination_reason": "Client disconnected",
                "terminated_by": "client_003",
                "final_status": "voluntary_disconnect",
                "capabilities": ["websocket", "video", "screen_share"],
                "metadata": {
                    "user_agent": "Desktop App v1.5",
                    "ip_address": "192.168.1.102",
                    "session_duration": "1h 15m",
                    "last_activity": "2025-07-11T22:10:45Z"
                },
                "record_created_at": firestore.SERVER_TIMESTAMP
            }
        ]

        # Add the documents to the collection
        collection_ref = db.collection(collection_name)
        logger.info(f"Creating '{collection_name}' collection...")

        for doc_data in client_in_terminated_room_docs:
            doc_ref = collection_ref.document(doc_data["id"])
            doc_ref.set(doc_data)
            logger.info(f"Created client in terminated room document: {doc_data['id']}")

        print(f"Created '{collection_name}' collection in database '{database_name}' with {len(client_in_terminated_room_docs)} sample documents:")
        for i, doc in enumerate(client_in_terminated_room_docs, 1):
            print(f"{i}. Document: {doc}")
        logger.info(f"Successfully created '{collection_name}' collection with {len(client_in_terminated_room_docs)} sample documents")
        
    except Exception as e:
        logger.error(f"Error creating '{collection_name}' collection: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main() 