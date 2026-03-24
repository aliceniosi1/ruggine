use logging::cpu::log_cpu_usage;
use tokio::time::{self, Duration};
use tokio::net::TcpListener;
use std::error::Error;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::io::BufReader;
use tokio::io::AsyncBufReadExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use common::Message;
use serde_json;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    tokio::spawn(async {
        let mut interval = time::interval(Duration::from_secs(120));
        loop {
            interval.tick().await;
            if let Err(e) = log_cpu_usage().await {
                eprintln!("Errore logging CPU: {}", e);
            }
        }
    });

    let groups: Arc<Mutex<HashMap<String, Vec<Arc<Mutex<OwnedWriteHalf>>>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let users: Arc<Mutex<HashMap<String, Arc<Mutex<OwnedWriteHalf>>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let invites: Arc<Mutex<HashMap<String, HashSet<String>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, addr) = listener.accept().await?;
        let (reader, writer) = stream.into_split();
        let writer = Arc::new(Mutex::new(writer));
        let groups = Arc::clone(&groups);
        let users = Arc::clone(&users);
        let invites = Arc::clone(&invites);
        tokio::spawn(async move {
            if let Err(e) = handle_client(reader, writer, addr, groups, users, invites).await {
                eprintln!("Errore con client {}: {}", addr, e);
            }
        });
    }
}

async fn handle_client(
    reader: OwnedReadHalf,
    writer: Arc<Mutex<OwnedWriteHalf>>,
    addr: SocketAddr,
    groups: Arc<Mutex<HashMap<String, Vec<Arc<Mutex<OwnedWriteHalf>>>>>>,
    users: Arc<Mutex<HashMap<String, Arc<Mutex<OwnedWriteHalf>>>>>,
    invites: Arc<Mutex<HashMap<String, HashSet<String>>>>,
) -> Result<(), Box<dyn Error>> {
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();
    loop {
        line.clear();
        let n = buf_reader.read_line(&mut line).await?;
        if n == 0 {
            println!("Client {} disconnesso", addr);
            break;
        }
        let raw = line.trim_end();
        println!("🚀 Ricevuto raw JSON: '{}'", raw);
        match serde_json::from_str::<Message>(raw) {
            Ok(Message::Login { username }) => {
                let mut u = users.lock().await;
                u.insert(username.clone(), writer.clone());
                drop(u);
                let cmds = "/create <gruppo>\n /invite <gruppo> <user>\n/join <gruppo>\n/leave <gruppo>\n/msg <gruppo> <testo>";
                let mut help = serde_json::to_string(&Message::Text {
                    group: "system".to_string(),
                    from: "server".to_string(),
                    content: cmds.to_string(),
                }).unwrap();
                help.push('\n');
                writer.lock().await.write_all(help.as_bytes()).await?;
                let pending = {
                    let inv_map = invites.lock().await;
                    inv_map.iter()
                        .filter(|(_, userset)| userset.contains(&username))
                        .map(|(grp, _)| grp.clone())
                        .collect::<Vec<_>>()
                };
                for grp in pending {
                    let mut inv_msg = serde_json::to_string(&Message::Invite {
                        group: grp.clone(),
                        user: username.clone(),
                    }).unwrap();
                    inv_msg.push('\n');
                    writer.lock().await.write_all(inv_msg.as_bytes()).await?;
                }
            }
            Ok(Message::Create { group }) => {
                // trova chi è il creatore
                let creator_opt = {
                    let u = users.lock().await;
                    u.iter()
                        .find_map(|(name, w)| if Arc::ptr_eq(w, &writer) { Some(name.clone()) } else { None })
                };
                if let Some(creator) = creator_opt {
                    println!("[Server][Create] group='{}' created by '{}'", group, creator);
                    // aggiungi subito creator come invitato e membro
                    {
                        let mut inv = invites.lock().await;
                        inv.entry(group.clone()).or_default().insert(creator.clone());
                    }
                    {
                        let mut g = groups.lock().await;
                        let entry = g.entry(group.clone()).or_default();
                        if !entry.iter().any(|w| Arc::ptr_eq(w, &writer)) {
                            entry.push(writer.clone());
                        }
                    }
                } else {
                    println!("[Server][Create] group='{}' created by unknown writer", group);
                }
                let mut ack = serde_json::to_string(&Message::Ack).unwrap();
                ack.push('\n');
                writer.lock().await.write_all(ack.as_bytes()).await?;
            }
            Ok(Message::Join { username, group }) => {
                let allowed = {
                    let inv = invites.lock().await;
                    inv.get(&group).map_or(false, |set| set.contains(&username))
                };
                println!("[Server][Join] user='{}' joining group='{}' allowed={}", username, group, allowed);
                if !allowed {
                    let mut err = serde_json::to_string(&Message::Error {
                        reason: format!("Impossibile unirsi: non sei stato invitato al gruppo '{}'", group)
                    }).unwrap();
                    err.push('\n');
                    writer.lock().await.write_all(err.as_bytes()).await?;
                } else {
                    let mut g = groups.lock().await;
                    let entry = g.entry(group.clone()).or_default();
                    if !entry.iter().any(|w| Arc::ptr_eq(w, &writer)) {
                        entry.push(writer.clone());
                    }
                    let mut ack = serde_json::to_string(&Message::Ack).unwrap();
                    ack.push('\n');
                    writer.lock().await.write_all(ack.as_bytes()).await?;
                    let mut u = users.lock().await;
                    u.insert(username.clone(), writer.clone());
                }
            }
            Ok(Message::Leave { group }) => {
                // Trova username
                let username = {
                    let u = users.lock().await;
                    u.iter()
                        .find_map(|(name, w)| if Arc::ptr_eq(w, &writer) { Some(name.clone()) } else { None })
                        .unwrap_or("qualcuno".to_string())
                };

                let mut g = groups.lock().await;
                if let Some(vec) = g.get_mut(&group) {
                    // Rimuovi l'utente
                    vec.retain(|w| !Arc::ptr_eq(w, &writer));

                    // Manda messaggio "utente ha lasciato il gruppo"
                    let msg = serde_json::to_string(&Message::Text {
                        group: group.clone(),
                        from: "system".to_string(),
                        content: format!("{} ha lasciato il gruppo", username),
                    }).unwrap() + "\n";

                    for peer in vec {
                        let mut w = peer.lock().await;
                        let _ = w.write_all(msg.as_bytes()).await;
                    }
                }

                let mut ack = serde_json::to_string(&Message::Ack).unwrap();
                ack.push('\n');
                writer.lock().await.write_all(ack.as_bytes()).await?;
            }
            Ok(Message::Text { group, from, content }) => {
                let writers_opt = {
                    let guard = groups.lock().await;
                    guard.get(&group).cloned()
                };
                let allowed = if let Some(writers) = writers_opt {
                    let ulock = users.lock().await;
                    writers.iter().any(|w| {
                        if let Some(uw) = ulock.get(&from) {
                            Arc::ptr_eq(uw, w)
                        } else { false }
                    })
                } else { false };
                if !allowed {
                    let mut err = serde_json::to_string(&Message::Error {
                        reason: format!("Impossibile inviare: non sei membro del gruppo '{}'", group)
                    }).unwrap();
                    err.push('\n');
                    writer.lock().await.write_all(err.as_bytes()).await?;
                } else {
                    let g2 = groups.lock().await;
                    if let Some(peers) = g2.get(&group) {
                        let mut out = serde_json::to_string(&Message::Text {
                            group: group.clone(),
                            from: from.clone(),
                            content: content.clone(),
                        }).unwrap();
                        out.push('\n');
                        for peer in peers {
                            let mut w = peer.lock().await;
                            let _ = w.write_all(out.as_bytes()).await;
                        }
                    }
                }
            }
            Ok(Message::Invite { group, user }) => {
                let inviter_opt = {
                    let ulock = users.lock().await;
                    ulock.iter()
                        .find_map(|(u, w)| if Arc::ptr_eq(w, &writer) { Some(u.clone()) } else { None })
                };
                let is_member = if let Some(inviter) = inviter_opt {
                    let uw_opt = {
                        let ulock = users.lock().await;
                        ulock.get(&inviter).cloned()
                    };
                    let members_opt = {
                        let guard = groups.lock().await;
                        guard.get(&group).cloned()
                    };
                    if let (Some(uw), Some(members)) = (uw_opt, members_opt) {
                        members.iter().any(|w| Arc::ptr_eq(w, &uw))
                    } else { false }
                } else { false };
                if !is_member {
                    let mut err = serde_json::to_string(&Message::Error {
                        reason: format!("Autorizzazione negata: non sei membro del gruppo '{}'", group)
                    }).unwrap();
                    err.push('\n');
                    writer.lock().await.write_all(err.as_bytes()).await?;
                } else {
                    let mut inv_map = invites.lock().await;
                    inv_map.entry(group.clone()).or_default().insert(user.clone());
                    let ulock = users.lock().await;
                    if let Some(writer) = ulock.get(&user) {
                        let mut out = serde_json::to_string(&Message::Invite {
                            group: group.clone(),
                            user: user.clone(),
                        }).unwrap();
                        out.push('\n');
                        let _ = writer.lock().await.write_all(out.as_bytes()).await;
                    }
                }
            }
            Ok(other) => {
                eprintln!("⚠️ Messaggio non gestito: {:?}", other);
            }
            Err(e) => {
                eprintln!("❌ Errore parsing JSON da {}: {}", addr, e);
            }
        }
    }
    Ok(())
}
