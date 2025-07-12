use firestore::paths;
use firestore::FirestoreDb;
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::config::Config;
use crate::database::error::DatabaseError;
use crate::database::models::{WebRTCClient, WebRTCClientRegistrationPayload, WebRTCClientStatus, ClientRole};
use crate::database::webrtc_client_repository::WebRTCClientRepository;

const COLLECTION_NAME: &str = "webrtc_clients";

pub struct FirestoreWebRTCClientRepository {
    db: FirestoreDb,
    _collection_name: String,
}

impl FirestoreWebRTCClientRepository {
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
impl WebRTCClientRepository for FirestoreWebRTCClientRepository {
    async fn register_client(&self, payload: WebRTCClientRegistrationPayload) -> Result<WebRTCClient, DatabaseError> {
        let client = WebRTCClient::new(
            payload.client_id,
            payload.room_id,
            payload.role,
            payload.session_id,
            payload.metadata,
        );
        
        let doc_id = client.client_id.clone();
        
        match self.db.fluent()
            .insert()
            .into(COLLECTION_NAME)
            .document_id(&doc_id)
            .object(&client)
            .execute::<WebRTCClient>()
            .await {
            Ok(created_client) => {
                info!("Registered WebRTC client: {}", doc_id);
                Ok(created_client)
            }
            Err(e) => {
                error!("Failed to register WebRTC client: {}", e);
                Err(DatabaseError::Write(format!("Failed to register WebRTC client: {e}")))
            }
        }
    }

    async fn get_client_by_id(&self, client_id: &str) -> Result<Option<WebRTCClient>, DatabaseError> {
        let result = self.db.fluent()
            .select()
            .by_id_in(COLLECTION_NAME)
            .obj::<WebRTCClient>()
            .one(client_id)
            .await;
        match result {
            Ok(client) => {
                debug!("Found WebRTC client: {}", client_id);
                Ok(client)
            }
            Err(e) => {
                let msg = format!("{e}");
                if msg.contains("not found") || msg.contains("NotFound") {
                    debug!("WebRTC client not found: {}", client_id);
                    Ok(None)
                } else {
                    error!("Failed to get WebRTC client: {}", e);
                    Err(DatabaseError::Read(format!("Failed to get WebRTC client: {e}")))
                }
            }
        }
    }

    async fn get_clients_by_room_id(&self, room_id: &str) -> Result<Vec<WebRTCClient>, DatabaseError> {
        let query = self.db.fluent()
            .select()
            .from(COLLECTION_NAME)
            .filter(|q| q.field("room_id").eq(room_id))
            .obj::<WebRTCClient>()
            .query();

        match query.await {
            Ok(clients) => {
                debug!("Found {} clients in room {}", clients.len(), room_id);
                Ok(clients)
            }
            Err(e) => {
                error!("Failed to get clients by room ID: {}", e);
                Err(DatabaseError::Read(format!("Failed to get clients by room ID: {e}")))
            }
        }
    }

    async fn get_clients_by_role(&self, room_id: &str, role: ClientRole) -> Result<Vec<WebRTCClient>, DatabaseError> {
        let role_str = match role {
            ClientRole::Sender => "sender",
            ClientRole::Receiver => "receiver",
        };

        let query = self.db.fluent()
            .select()
            .from(COLLECTION_NAME)
            .filter(|q| q.field("room_id").eq(room_id).and(Some(q.field("role").eq(role_str).expect("role filter must be valid"))))
            .obj::<WebRTCClient>()
            .query();

        match query.await {
            Ok(clients) => {
                debug!("Found {} {} clients in room: {}", clients.len(), role_str, room_id);
                Ok(clients)
            }
            Err(e) => {
                error!("Failed to get clients by role: {}", e);
                Err(DatabaseError::Read(format!("Failed to get clients by role: {e}")))
            }
        }
    }

    async fn update_client_status(&self, client_id: &str, status: WebRTCClientStatus) -> Result<(), DatabaseError> {
        let client = match self.get_client_by_id(client_id).await? {
            Some(client) => client,
            None => return Err(DatabaseError::NotFound("Client not found".to_string())),
        };

        let mut updated_client = client;
        updated_client.status = status.clone();
        let status_for_log = status;
        
        match self.db.fluent()
            .update()
            .fields(paths!(WebRTCClient::status))
            .in_col(COLLECTION_NAME)
            .document_id(client_id)
            .object(&updated_client)
            .execute::<WebRTCClient>()
            .await {
            Ok(_) => {
                info!("Updated client status: {} -> {:?}", client_id, status_for_log);
                Ok(())
            }
            Err(e) => {
                error!("Failed to update client status: {}", e);
                Err(DatabaseError::Write(format!("Failed to update client status: {e}")))
            }
        }
    }

    async fn set_session_id(&self, client_id: &str, session_id: &str) -> Result<(), DatabaseError> {
        let client = match self.get_client_by_id(client_id).await? {
            Some(client) => client,
            None => return Err(DatabaseError::NotFound("Client not found".to_string())),
        };

        let mut updated_client = client;
        updated_client.session_id = Some(session_id.to_string());
        
        match self.db.fluent()
            .update()
            .fields(paths!(WebRTCClient::session_id))
            .in_col(COLLECTION_NAME)
            .document_id(client_id)
            .object(&updated_client)
            .execute::<WebRTCClient>()
            .await {
            Ok(_) => {
                info!("Set session ID: {} -> {}", client_id, session_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to set session ID: {}", e);
                Err(DatabaseError::Write(format!("Failed to set session ID: {e}")))
            }
        }
    }

    async fn get_client_by_session_id(&self, session_id: &str) -> Result<Option<WebRTCClient>, DatabaseError> {
        let query = self.db.fluent()
            .select()
            .from(COLLECTION_NAME)
            .filter(|q| q.field("session_id").eq(session_id))
            .obj::<WebRTCClient>()
            .query();

        match query.await {
            Ok(clients) => {
                if clients.is_empty() {
                    debug!("Client not found by session ID: {}", session_id);
                    Ok(None)
                } else {
                    debug!("Found client by session ID: {}", session_id);
                    Ok(Some(clients[0].clone()))
                }
            }
            Err(e) => {
                error!("Failed to get client by session ID: {}", e);
                Err(DatabaseError::Read(format!("Failed to get client by session ID: {e}")))
            }
        }
    }

    async fn get_active_clients(&self) -> Result<Vec<WebRTCClient>, DatabaseError> {
        let query = self.db.fluent()
            .select()
            .from(COLLECTION_NAME)
            .filter(|q| q.field("status").eq("Active"))
            .obj::<WebRTCClient>()
            .query();

        match query.await {
            Ok(clients) => {
                debug!("Found {} active clients", clients.len());
                Ok(clients)
            }
            Err(e) => {
                error!("Failed to get active clients: {}", e);
                Err(DatabaseError::Read(format!("Failed to get active clients: {e}")))
            }
        }
    }

    async fn get_active_clients_in_room(&self, room_id: &str) -> Result<Vec<WebRTCClient>, DatabaseError> {
        let query = self.db.fluent()
            .select()
            .from(COLLECTION_NAME)
            .filter(|q| q.field("room_id").eq(room_id).and(Some(q.field("status").eq("Active").expect("status filter must be valid"))))
            .obj::<WebRTCClient>()
            .query();

        match query.await {
            Ok(clients) => {
                debug!("Found {} active clients in room: {}", clients.len(), room_id);
                Ok(clients)
            }
            Err(e) => {
                error!("Failed to get active clients in room: {}", e);
                Err(DatabaseError::Read(format!("Failed to get active clients in room: {e}")))
            }
        }
    }

    async fn disconnect_client(&self, client_id: &str, reason: &str) -> Result<(), DatabaseError> {
        let client = match self.get_client_by_id(client_id).await? {
            Some(client) => client,
            None => return Err(DatabaseError::NotFound("Client not found".to_string())),
        };

        let mut updated_client = client;
        updated_client.status = WebRTCClientStatus::Disconnected;
        
        match self.db.fluent()
            .update()
            .fields(paths!(WebRTCClient::status))
            .in_col(COLLECTION_NAME)
            .document_id(client_id)
            .object(&updated_client)
            .execute::<WebRTCClient>()
            .await {
            Ok(_) => {
                info!("Disconnected client: {} (reason: {})", client_id, reason);
                Ok(())
            }
            Err(e) => {
                error!("Failed to disconnect client: {}", e);
                Err(DatabaseError::Write(format!("Failed to disconnect client: {e}")))
            }
        }
    }

    async fn remove_client_from_room(&self, client_id: &str, _room_id: &str) -> Result<(), DatabaseError> {
        let client = match self.get_client_by_id(client_id).await? {
            Some(client) => client,
            None => return Err(DatabaseError::NotFound("Client not found".to_string())),
        };

        let mut updated_client = client;
        updated_client.room_id = String::new();
        
        match self.db.fluent()
            .update()
            .fields(paths!(WebRTCClient::room_id))
            .in_col(COLLECTION_NAME)
            .document_id(client_id)
            .object(&updated_client)
            .execute::<WebRTCClient>()
            .await {
            Ok(_) => {
                info!("Removed client {} from room", client_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to remove client from room: {}", e);
                Err(DatabaseError::Write(format!("Failed to remove client from room: {e}")))
            }
        }
    }

    async fn delete_client(&self, client_id: &str) -> Result<(), DatabaseError> {
        match self.db.fluent()
            .delete()
            .from(COLLECTION_NAME)
            .document_id(client_id)
            .execute()
            .await {
            Ok(_) => {
                info!("Deleted WebRTC client: {}", client_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to delete WebRTC client: {}", e);
                Err(DatabaseError::Write(format!("Failed to delete WebRTC client: {e}")))
            }
        }
    }

    async fn get_client_count(&self) -> Result<usize, DatabaseError> {
        let query = self.db.fluent()
            .select()
            .from(COLLECTION_NAME)
            .obj::<WebRTCClient>()
            .query();

        match query.await {
            Ok(clients) => {
                debug!("Total client count: {}", clients.len());
                Ok(clients.len())
            }
            Err(e) => {
                error!("Failed to get client count: {}", e);
                Err(DatabaseError::Read(format!("Failed to get client count: {e}")))
            }
        }
    }

    async fn get_client_count_in_room(&self, room_id: &str) -> Result<usize, DatabaseError> {
        let query = self.db.fluent()
            .select()
            .from(COLLECTION_NAME)
            .filter(|q| q.field("room_id").eq(room_id))
            .obj::<WebRTCClient>()
            .query();

        match query.await {
            Ok(clients) => {
                debug!("Client count in room {}: {}", room_id, clients.len());
                Ok(clients.len())
            }
            Err(e) => {
                error!("Failed to get client count in room: {}", e);
                Err(DatabaseError::Read(format!("Failed to get client count in room: {e}")))
            }
        }
    }
} 