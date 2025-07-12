// This file is kept for backward compatibility
// Repository traits have been moved to separate files:
// - client_repository.rs: ClientRepository trait
// - terminated_room_repository.rs: TerminatedRoomRepository trait  
// - room_created_repository.rs: RoomCreatedRepository trait
// - client_in_room_repository.rs: ClientInRoomRepository trait
// - client_in_terminated_room_repository.rs: ClientInTerminatedRoomRepository trait
// - repository_factory.rs: RepositoryFactory trait

// Re-export all repository traits for backward compatibility
pub use crate::database::client_repository::ClientRepository;
pub use crate::database::terminated_room_repository::TerminatedRoomRepository;
pub use crate::database::room_created_repository::RoomCreatedRepository;
pub use crate::database::client_in_room_repository::ClientInRoomRepository;
pub use crate::database::client_in_terminated_room_repository::ClientInTerminatedRoomRepository;
pub use crate::database::repository_factory::RepositoryFactory; 