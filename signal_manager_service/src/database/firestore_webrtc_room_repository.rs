use firestore::paths;
use firestore::FirestoreDb;
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::config::Config;
use crate::database::error::DatabaseError;
use crate::database::models::{WebRTCRoom, WebRTCRoomCreationPayload, WebRTCRoomStatus};
use crate::database::webrtc_room_repository::WebRTCRoomRepository;

const COLLECTION_NAME: &str = "webrtc_rooms";

pub struct FirestoreWebRTCRoomRepository {
    db: FirestoreDb,
    _collection_name: String,
}

impl FirestoreWebRTCRoomRepository {
    pub async fn new(config: Arc<Config>) -> Result<Self, DatabaseError> {
        let db = FirestoreDb::new(&config.gcp.project_id)
            .await
            .map_err(|e| DatabaseError::Connection(format!("Failed to create Firestore client: {e}")))?;
        
        Ok(Self {
            db,
            _collection_name: COLLECTION_NAME.to_string(),
        })
    }
}

#[async_trait::async_trait]
impl WebRTCRoomRepository for FirestoreWebRTCRoomRepository {
    async fn create_room(&self, payload: WebRTCRoomCreationPayload) -> Result<WebRTCRoom, DatabaseError> {
        let room = WebRTCRoom::new(
            payload.room_id,
            payload.app_id,
            payload.sender_client_id,
            payload.receiver_client_id,
            payload.session_id,
            payload.metadata,
        );
        
        let doc_id = room.room_id.clone();
        
        match self.db.fluent()
            .insert()
            .into(COLLECTION_NAME)
            .document_id(&doc_id)
            .object(&room)
            .execute::<WebRTCRoom>()
            .await {
            Ok(created_room) => {
                info!("Created WebRTC room: {}", doc_id);
                Ok(created_room)
            }
            Err(e) => {
                error!("Failed to create WebRTC room: {}", e);
                Err(DatabaseError::Write(format!("Failed to create WebRTC room: {e}")))
            }
        }
    }

    async fn get_room_by_id(&self, room_id: &str) -> Result<Option<WebRTCRoom>, DatabaseError> {
        let result = self.db.fluent()
            .select()
            .by_id_in(COLLECTION_NAME)
            .obj::<WebRTCRoom>()
            .one(room_id)
            .await;
        match result {
            Ok(room) => {
                debug!("Found WebRTC room: {}", room_id);
                Ok(room)
            }
            Err(e) => {
                let msg = format!("{e}");
                if msg.contains("not found") || msg.contains("NotFound") {
                    debug!("WebRTC room not found: {}", room_id);
                    Ok(None)
                } else {
                    error!("Failed to get WebRTC room: {}", e);
                    Err(DatabaseError::Read(format!("Failed to get WebRTC room: {e}")))
                }
            }
        }
    }

    async fn get_room_by_uuid(&self, room_uuid: &str) -> Result<Option<WebRTCRoom>, DatabaseError> {
        let query = self.db.fluent()
            .select()
            .from(COLLECTION_NAME)
            .filter(|q| q.field("room_id").eq(room_uuid))
            .obj::<WebRTCRoom>()
            .query();

        match query.await {
            Ok(rooms) => {
                if rooms.is_empty() {
                    debug!("WebRTC room not found by UUID: {}", room_uuid);
                    Ok(None)
                } else {
                    debug!("Found WebRTC room by UUID: {}", room_uuid);
                    Ok(Some(rooms[0].clone()))
                }
            }
            Err(e) => {
                error!("Failed to get WebRTC room by UUID: {}", e);
                Err(DatabaseError::Read(format!("Failed to get WebRTC room by UUID: {e}")))
            }
        }
    }

    async fn update_room_status(&self, room_id: &str, status: WebRTCRoomStatus) -> Result<(), DatabaseError> {
        let room = match self.get_room_by_id(room_id).await? {
            Some(room) => room,
            None => return Err(DatabaseError::NotFound("Room not found".to_string())),
        };

        let mut updated_room = room;
        updated_room.status = status.clone();
        let status_for_log = status;
        
        match self.db.fluent()
            .update()
            .fields(paths!(WebRTCRoom::status))
            .in_col(COLLECTION_NAME)
            .document_id(room_id)
            .object(&updated_room)
            .execute::<WebRTCRoom>()
            .await {
            Ok(_) => {
                info!("Updated room status: {} -> {:?}", room_id, status_for_log);
                Ok(())
            }
            Err(e) => {
                error!("Failed to update room status: {}", e);
                Err(DatabaseError::Write(format!("Failed to update room status: {e}")))
            }
        }
    }

    async fn set_sender_client_id(&self, room_id: &str, client_id: &str) -> Result<(), DatabaseError> {
        let room = match self.get_room_by_id(room_id).await? {
            Some(room) => room,
            None => return Err(DatabaseError::NotFound("Room not found".to_string())),
        };

        let mut updated_room = room;
        updated_room.sender_client_id = Some(client_id.to_string());
        
        match self.db.fluent()
            .update()
            .fields(paths!(WebRTCRoom::sender_client_id))
            .in_col(COLLECTION_NAME)
            .document_id(room_id)
            .object(&updated_room)
            .execute::<WebRTCRoom>()
            .await {
            Ok(_) => {
                info!("Set sender client ID: {} -> {}", room_id, client_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to set sender client ID: {}", e);
                Err(DatabaseError::Write(format!("Failed to set sender client ID: {e}")))
            }
        }
    }

    async fn set_receiver_client_id(&self, room_id: &str, client_id: &str) -> Result<(), DatabaseError> {
        let room = match self.get_room_by_id(room_id).await? {
            Some(room) => room,
            None => return Err(DatabaseError::NotFound("Room not found".to_string())),
        };

        let mut updated_room = room;
        updated_room.receiver_client_id = Some(client_id.to_string());
        
        match self.db.fluent()
            .update()
            .fields(paths!(WebRTCRoom::receiver_client_id))
            .in_col(COLLECTION_NAME)
            .document_id(room_id)
            .object(&updated_room)
            .execute::<WebRTCRoom>()
            .await {
            Ok(_) => {
                info!("Set receiver client ID: {} -> {}", room_id, client_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to set receiver client ID: {}", e);
                Err(DatabaseError::Write(format!("Failed to set receiver client ID: {e}")))
            }
        }
    }

    async fn set_session_id(&self, room_id: &str, session_id: &str) -> Result<(), DatabaseError> {
        let room = match self.get_room_by_id(room_id).await? {
            Some(room) => room,
            None => return Err(DatabaseError::NotFound("Room not found".to_string())),
        };

        let mut updated_room = room;
        updated_room.session_id = Some(session_id.to_string());
        
        match self.db.fluent()
            .update()
            .fields(paths!(WebRTCRoom::session_id))
            .in_col(COLLECTION_NAME)
            .document_id(room_id)
            .object(&updated_room)
            .execute::<WebRTCRoom>()
            .await {
            Ok(_) => {
                info!("Set session ID: {} -> {}", room_id, session_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to set session ID: {}", e);
                Err(DatabaseError::Write(format!("Failed to set session ID: {e}")))
            }
        }
    }

    async fn get_active_rooms(&self) -> Result<Vec<WebRTCRoom>, DatabaseError> {
        let query = self.db.fluent()
            .select()
            .from(COLLECTION_NAME)
            .filter(|q| q.field("status").eq("Active"))
            .obj::<WebRTCRoom>()
            .query();

        match query.await {
            Ok(rooms) => {
                debug!("Found {} active rooms", rooms.len());
                Ok(rooms)
            }
            Err(e) => {
                error!("Failed to get active rooms: {}", e);
                Err(DatabaseError::Read(format!("Failed to get active rooms: {e}")))
            }
        }
    }

    async fn get_rooms_by_client_id(&self, client_id: &str) -> Result<Vec<WebRTCRoom>, DatabaseError> {
        let query = self.db.fluent()
            .select()
            .from(COLLECTION_NAME)
            .filter(|q| q.field("sender_client_id").eq(client_id).or(Some(q.field("receiver_client_id").eq(client_id).expect("receiver_client_id filter must be valid"))))
            .obj::<WebRTCRoom>()
            .query();

        match query.await {
            Ok(rooms) => {
                debug!("Found {} rooms for client {}", rooms.len(), client_id);
                Ok(rooms)
            }
            Err(e) => {
                error!("Failed to get rooms by client ID: {}", e);
                Err(DatabaseError::Read(format!("Failed to get rooms by client ID: {e}")))
            }
        }
    }

    async fn terminate_room(&self, room_id: &str, reason: &str) -> Result<(), DatabaseError> {
        let room = match self.get_room_by_id(room_id).await? {
            Some(room) => room,
            None => return Err(DatabaseError::NotFound("Room not found".to_string())),
        };

        let mut updated_room = room;
        updated_room.status = WebRTCRoomStatus::Terminated;
        
        match self.db.fluent()
            .update()
            .fields(paths!(WebRTCRoom::status))
            .in_col(COLLECTION_NAME)
            .document_id(room_id)
            .object(&updated_room)
            .execute::<WebRTCRoom>()
            .await {
            Ok(_) => {
                info!("Terminated room: {} (reason: {})", room_id, reason);
                Ok(())
            }
            Err(e) => {
                error!("Failed to terminate room: {}", e);
                Err(DatabaseError::Write(format!("Failed to terminate room: {e}")))
            }
        }
    }

    async fn delete_room(&self, room_id: &str) -> Result<(), DatabaseError> {
        match self.db.fluent()
            .delete()
            .from(COLLECTION_NAME)
            .document_id(room_id)
            .execute()
            .await {
            Ok(_) => {
                info!("Deleted WebRTC room: {}", room_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to delete WebRTC room: {}", e);
                Err(DatabaseError::Write(format!("Failed to delete WebRTC room: {e}")))
            }
        }
    }

    async fn get_room_count(&self) -> Result<usize, DatabaseError> {
        let query = self.db.fluent()
            .select()
            .from(COLLECTION_NAME)
            .obj::<WebRTCRoom>()
            .query();

        match query.await {
            Ok(rooms) => {
                debug!("Total room count: {}", rooms.len());
                Ok(rooms.len())
            }
            Err(e) => {
                error!("Failed to get room count: {}", e);
                Err(DatabaseError::Read(format!("Failed to get room count: {e}")))
            }
        }
    }
} 