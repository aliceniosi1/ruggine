use serde::{Serialize, Deserialize};

/// Messaggi che client e server si scambiano
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Login { username: String },
    /// Un utente si unisce a un gruppo
    Join { username: String, group: String },
    /// Invita un utente in un gruppo
    Invite { group: String, user: String },
    /// Testo inviato in un gruppo
    Text { group: String, from: String, content: String },
    /// Acknowledgement generico
    Ack,
    /// Errore con motivazione
    Error { reason: String },
    Create { group: String },
    Leave  { group: String },
}