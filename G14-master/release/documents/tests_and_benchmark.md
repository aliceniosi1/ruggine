# Esempi di test e benchmark adottati

## ✅ Test funzionali manuali

### 🔹 Prima fase con client terminale
- Client da terminale (senza GUI) usato come prima implementazione.
- Comandi supportati:
  ```
  /create G1
  /invite bob G1
  /join G1
  /msg G1 Hello
  /leave G1
  ```
- Utile per verificare velocemente la correttezza del server e del protocollo JSON line-based.

### 🔹 GUI: login e scambio messaggi
- Avviato il server con:
  ```
  cargo run --bin server
  ```
- Avviati due client GUI con:
  ```
  cargo run --bin gui_client
  ```
- Login effettuato con utenti `alice` e `bob`.
- `alice` crea un gruppo `G1` e invita `bob`.
- `bob` clicca su `Join Group` e inizia lo scambio di messaggi con `alice`.
- Verificato che i messaggi appaiono in tempo reale in entrambi i client.

### 🔹 Invite / Join / Leave
- Un utente invitato vede comparire il gruppo nella lista ma deve premere `Join Group` per accedere.
- Dopo `Leave Group`, il gruppo sparisce dalla lista.
- Gli altri membri ricevono un messaggio `<system>: alice ha lasciato il gruppo`.

### 🔹 Tre utenti nello stesso gruppo
- Login con `alice`, `bob` e `charlie`.
- `alice` crea un gruppo `G2` e invita `bob` e `charlie`.
- Entrambi accettano l'invito con `Join Group`.
- `alice` decide di fare `Leave Group`.
- Verificato che `bob` e `charlie` continuano a scambiarsi messaggi, mentre `alice` non riceve più nulla e non vede il gruppo.
### 🔹 Chat multiple nella GUI con utenti distinti
- Effettuato login simultaneo con:
  - `alice`
  - `bob`
  - `bibi`
  - `charlie`
- `alice` crea un gruppo con `bob`, `bibi` e `charlie` e inizia conversazioni multiple.
- `alice` crea anche un gruppo esclusivo con `charlie`.
- Verificato che:
  - I messaggi nelle diverse chat restano correttamente separati.
  - La lista dei gruppi nella GUI mantiene la distinzione e mostra lo storico corretto di ciascuna conversazione.
- Nessun errore né mescolamento dei messaggi osservato.

### 🔹 Rientro dopo leave con tentativo di join non autorizzato (Client Testuale)

#### 🔸 Scenario iniziale
- `alice` inserisce il proprio nome utente:
  Inserisci il tuo nome utente: alice

- `bob` inserisce il proprio nome utente:
  Inserisci il tuo nome utente: bob

- `charlie` inserisce il proprio nome utente:
  Inserisci il tuo nome utente: charlie


#### 🔸 Creazione gruppo e inviti
- `alice` crea il gruppo `G2` con:
  /create G2

- `alice` invita `bob` nel gruppo con:
  /invite G2 bob

- `alice` invita `charlie` nel gruppo con:
  /invite G2 charlie

#### 🔸 Join iniziale
- `bob` si unisce al gruppo con:
  /join G2

- `charlie` si unisce al gruppo con:
  /join G2

#### 🔸 Verifica scambio messaggi
- `alice`, `bob` e `charlie` inviano messaggi con:
  /msg G2 ciao a tutti
- Tutti i membri ricevono correttamente i messaggi.

#### 🔸 Leave e tentativo di rientro non autorizzato
- `charlie` lascia il gruppo con:
  /leave G2
- `charlie` tenta subito di rientrare con:
  /join G2
- Il server risponde con:
  Ricevuto: Error { reason: "Impossibile unirsi: non sei stato invitato al gruppo 'G2'" }

#### 🔸 Reinvitare e rientrare
- `alice` o `bob`(testati entrambi) reinvita `charlie` nel gruppo:
  /invite G2 charlie

- `charlie` riceve nuovamente l'invito e riprova a unirsi con:
  /join G2

- Il server invia `Ack` e `charlie` torna membro del gruppo.

#### 🔸 Verifica finale
- `charlie` invia:
  /msg G2 sono tornato!
- `alice` e `bob` ricevono il messaggio, confermando che il reinvito e il rejoin

### 🔹 Invite multipli e test di inviti errati
- `alice` prova a invitare un utente non connesso: nessun crash, l'invito rimane pending.
- Al login successivo l'utente riceve l'invito e può entrare nel gruppo.

## ✅ Test robustezza con più client
- Avviati **5 client GUI** simultanei.
- Tutti joinano lo stesso gruppo `StressTest`.
- Ogni client invia messaggi in modo rapido e simultaneo.
- Messaggi ricevuti ordinati, nessun crash osservato.


## 📌 Conclusioni sui test
- Il sistema ha superato i test funzionali manuali e le prove di robustezza con più client.
- La gestione asincrona (`tokio` + `mpsc`) garantisce reattività e basso carico anche con più utenti.
