use tokio::net::TcpStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use common::Message;
use serde_json;
use std::io::{stdin, stdout, Write};


// 1) Connetti al server
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Prompt iniziali per username e gruppo ---
    print!("Inserisci il tuo username: ");
    stdout().flush().unwrap();
    let mut username = String::new();
    stdin().read_line(&mut username)?;
    let username = username.trim().to_string();

    // default empty group (will be set via /join or /create)
    let mut group = String::new();
    // ----------------------------------------------
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    let (reader, mut writer) = stream.into_split();

    let recv_username = username.clone();

    // Trasforma il reader in BufReader per leggere linee
    let mut buf_reader = BufReader::new(reader);

    // 3a) Invia Login iniziale
    let login = Message::Login { username: username.clone() };
    let mut login_json = serde_json::to_string(&login)?;
    login_json.push('\n');
    writer.write_all(login_json.as_bytes()).await?;

    // 3b) Attendi e mostra la lista comandi inviata dal server
    let mut help_line = String::new();
    buf_reader.read_line(&mut help_line).await?;
    if let Ok(Message::Text { content, .. }) = serde_json::from_str::<Message>(&help_line) {
        println!("{}", content);
    }

    // 3c) Spawn task per ricevere tutti gli altri messaggi
    tokio::spawn(async move {
        let mut line = String::new();
        loop {
            line.clear();
            match buf_reader.read_line(&mut line).await {
                Ok(0) => break,
                Ok(_) => {
                    if let Ok(msg) = serde_json::from_str::<Message>(&line) {
                        match &msg {
                            Message::Text { from, .. } if from == &recv_username => {
                                println!("Inviato: {:?}", msg);
                            }
                            _ => {
                                println!("Ricevuto: {:?}", msg);
                            }
                        }
                    } else {
                        println!("Ricevuto (raw): {}", line.trim_end());
                    }
                }
                Err(e) => { eprintln!("Errore lettura: {}", e); break; }
            }
        }
    });

    // 4) Loop di invio: leggi da stdin e invia come Text
    let stdin_reader = BufReader::new(tokio::io::stdin());
    let mut lines = stdin_reader.lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let input = line.trim_end();

        // /msg <group> <text>
        if let Some(rest) = input.strip_prefix("/msg ") {
            if let Some((grp, msg_text)) = rest.split_once(' ') {
                let msg = Message::Text {
                    group: grp.to_string(),
                    from: username.clone(),
                    content: msg_text.to_string(),
                };
                let mut out = serde_json::to_string(&msg)?;
                out.push('\n');
                writer.write_all(out.as_bytes()).await?;

                continue;
            }
        }

        // /invite <group> <user>
        if let Some(rest) = input.strip_prefix("/invite ") {
            if let Some((grp, user)) = rest.split_once(' ') {
                let invite = Message::Invite {
                    group: grp.to_string(),
                    user: user.to_string(),
                };
                let mut out = serde_json::to_string(&invite)?;
                out.push('\n');
                writer.write_all(out.as_bytes()).await?;
                println!("Inviato: {:?}", invite);
                continue;
            }
        }

        // /join <group>
        if let Some(grp) = input.strip_prefix("/join ") {
            let newg = grp.to_string();
            group = newg.clone();
            let join = Message::Join {
                username: username.clone(),
                group: newg.clone(),
            };
            let mut out = serde_json::to_string(&join)?;
            out.push('\n');
            writer.write_all(out.as_bytes()).await?;
            println!("Inviato: {:?}", join);
            continue;
        }

        // /create <group>
        if let Some(grp) = input.strip_prefix("/create ") {
            let newg = grp.trim().to_string();
            // Invia Create, non Join, e non cambia ancora il gruppo attivo
            let create = Message::Create {
                group: newg.clone(),
            };
            let mut out = serde_json::to_string(&create)?;
            out.push('\n');
            writer.write_all(out.as_bytes()).await?;
            println!("Inviato (create): {:?}", create);
            continue;
        }

        // /leave <group>
        if let Some(grp) = input.strip_prefix("/leave ") {
            let leave = Message::Leave { group: grp.to_string() };
            let mut out = serde_json::to_string(&leave)?;
            out.push('\n');
            writer.write_all(out.as_bytes()).await?;
            println!("Inviato: {:?}", leave);
            // Se l'utente abbandona il gruppo, resetta il gruppo attivo
            group.clear();
            println!("Hai abbandonato il gruppo!");
            continue;
        }

        // fallback: testo libero
        let msg = Message::Text {
            group: group.clone(),
            from: username.clone(),
            content: input.to_string(),
        };
        let mut out = serde_json::to_string(&msg)?;
        out.push('\n');
        writer.write_all(out.as_bytes()).await?;
        println!("Inviato: {:?}", msg);
    }

    Ok(())
}