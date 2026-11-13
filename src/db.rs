//! Persistência em SQLite: esquema e consultas.

use std::path::{Path, PathBuf};

use chrono::{Duration, Local};
use rusqlite::{params, Connection};

use crate::model::{equipment_label, Client, Equipment, Payment, Quote, QuoteStatus};
use crate::util::today_iso;

pub type DbResult<T> = Result<T, String>;

fn db_err<E: std::fmt::Display>(err: E) -> String {
    err.to_string()
}

const SCHEMA: &str = r#"
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS clients (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    phone       TEXT NOT NULL DEFAULT '',
    email       TEXT NOT NULL DEFAULT '',
    address     TEXT NOT NULL DEFAULT '',
    notes       TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS equipment (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id   INTEGER NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    kind        TEXT NOT NULL,
    brand       TEXT NOT NULL DEFAULT '',
    model       TEXT NOT NULL DEFAULT '',
    serial      TEXT NOT NULL DEFAULT '',
    notes       TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS quotes (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id      INTEGER NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    equipment_id   INTEGER REFERENCES equipment(id) ON DELETE SET NULL,
    description    TEXT NOT NULL DEFAULT '',
    status         TEXT NOT NULL DEFAULT 'pendente',
    labor_cents    INTEGER NOT NULL DEFAULT 0,
    parts_cents    INTEGER NOT NULL DEFAULT 0,
    discount_cents INTEGER NOT NULL DEFAULT 0,
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS payments (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    quote_id     INTEGER NOT NULL REFERENCES quotes(id) ON DELETE CASCADE,
    amount_cents INTEGER NOT NULL,
    method       TEXT NOT NULL DEFAULT 'dinheiro',
    paid_on      TEXT NOT NULL,
    notes        TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_equipment_client ON equipment(client_id);
CREATE INDEX IF NOT EXISTS idx_quotes_client ON quotes(client_id);
CREATE INDEX IF NOT EXISTS idx_quotes_equipment ON quotes(equipment_id);
CREATE INDEX IF NOT EXISTS idx_payments_quote ON payments(quote_id);
PRAGMA user_version = 1;
"#;

/// Banco de dados da oficina.
pub struct Store {
    conn: Connection,
    pub path: PathBuf,
}

impl Store {
    /// Abre o banco no diretório de dados do usuário.
    pub fn open_default() -> DbResult<Self> {
        let path = data_dir().join("oficina.db");
        Self::open(&path)
    }

    /// Abre (ou cria) o banco no caminho informado.
    pub fn open(path: &Path) -> DbResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(db_err)?;
        }
        let conn = Connection::open(path).map_err(db_err)?;
        conn.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 3000;",
        )
        .map_err(db_err)?;
        let store = Self {
            conn,
            path: path.to_path_buf(),
        };
        store.conn.execute_batch(SCHEMA).map_err(db_err)?;
        Ok(store)
    }

    /// Banco em memória, útil para testes.
    #[cfg(test)]
    pub fn in_memory() -> DbResult<Self> {
        let conn = Connection::open_in_memory().map_err(db_err)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(db_err)?;
        let store = Self {
            conn,
            path: PathBuf::from(":memory:"),
        };
        store.conn.execute_batch(SCHEMA).map_err(db_err)?;
        Ok(store)
    }

    pub fn is_empty(&self) -> bool {
        self.count("clients") == 0
    }

    fn count(&self, table: &str) -> i64 {
        self.conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap_or(0)
    }

    // ---------------------------------------------------------------- Clientes

    pub fn list_clients(&self) -> DbResult<Vec<Client>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT c.id, c.name, c.phone, c.email, c.address, c.notes, c.created_at,
                        (SELECT COUNT(*) FROM equipment e WHERE e.client_id = c.id)
                 FROM clients c
                 ORDER BY c.name COLLATE NOCASE",
            )
            .map_err(db_err)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Client {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    phone: row.get(2)?,
                    email: row.get(3)?,
                    address: row.get(4)?,
                    notes: row.get(5)?,
                    created_at: row.get(6)?,
                    equipment_count: row.get(7)?,
                })
            })
            .map_err(db_err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(db_err)
    }

    pub fn insert_client(
        &self,
        name: &str,
        phone: &str,
        email: &str,
        address: &str,
        notes: &str,
    ) -> DbResult<i64> {
        self.conn
            .execute(
                "INSERT INTO clients (name, phone, email, address, notes, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![name, phone, email, address, notes, today_iso()],
            )
            .map_err(db_err)?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_client(
        &self,
        id: i64,
        name: &str,
        phone: &str,
        email: &str,
        address: &str,
        notes: &str,
    ) -> DbResult<()> {
        self.conn
            .execute(
                "UPDATE clients SET name = ?2, phone = ?3, email = ?4, address = ?5, notes = ?6
                 WHERE id = ?1",
                params![id, name, phone, email, address, notes],
            )
            .map_err(db_err)?;
        Ok(())
    }

    pub fn delete_client(&self, id: i64) -> DbResult<()> {
        self.conn
            .execute("DELETE FROM clients WHERE id = ?1", params![id])
            .map_err(db_err)?;
        Ok(())
    }

    // ------------------------------------------------------------ Equipamentos

    pub fn list_equipment(&self) -> DbResult<Vec<Equipment>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT e.id, e.client_id, e.kind, e.brand, e.model, e.serial, e.notes,
                        e.created_at, c.name,
                        (SELECT COUNT(*) FROM quotes q WHERE q.equipment_id = e.id)
                 FROM equipment e
                 JOIN clients c ON c.id = e.client_id
                 ORDER BY c.name COLLATE NOCASE, e.kind COLLATE NOCASE",
            )
            .map_err(db_err)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Equipment {
                    id: row.get(0)?,
                    client_id: row.get(1)?,
                    kind: row.get(2)?,
                    brand: row.get(3)?,
                    model: row.get(4)?,
                    serial: row.get(5)?,
                    notes: row.get(6)?,
                    created_at: row.get(7)?,
                    client_name: row.get(8)?,
                    quote_count: row.get(9)?,
                })
            })
            .map_err(db_err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(db_err)
    }

    pub fn insert_equipment(
        &self,
        client_id: i64,
        kind: &str,
        brand: &str,
        model: &str,
        serial: &str,
        notes: &str,
    ) -> DbResult<i64> {
        self.conn
            .execute(
                "INSERT INTO equipment (client_id, kind, brand, model, serial, notes, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![client_id, kind, brand, model, serial, notes, today_iso()],
            )
            .map_err(db_err)?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_equipment(
        &self,
        id: i64,
        client_id: i64,
        kind: &str,
        brand: &str,
        model: &str,
        serial: &str,
        notes: &str,
    ) -> DbResult<()> {
        self.conn
            .execute(
                "UPDATE equipment
                 SET client_id = ?2, kind = ?3, brand = ?4, model = ?5, serial = ?6, notes = ?7
                 WHERE id = ?1",
                params![id, client_id, kind, brand, model, serial, notes],
            )
            .map_err(db_err)?;
        Ok(())
    }

    pub fn delete_equipment(&self, id: i64) -> DbResult<()> {
        self.conn
            .execute("DELETE FROM equipment WHERE id = ?1", params![id])
            .map_err(db_err)?;
        Ok(())
    }

    // -------------------------------------------------------------- Orçamentos

    pub fn list_quotes(&self) -> DbResult<Vec<Quote>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT q.id, q.client_id, q.equipment_id, q.description, q.status,
                        q.labor_cents, q.parts_cents, q.discount_cents, q.created_at,
                        c.name, e.kind, e.brand, e.model,
                        COALESCE((SELECT SUM(p.amount_cents) FROM payments p WHERE p.quote_id = q.id), 0)
                 FROM quotes q
                 JOIN clients c ON c.id = q.client_id
                 LEFT JOIN equipment e ON e.id = q.equipment_id
                 ORDER BY q.created_at DESC, q.id DESC",
            )
            .map_err(db_err)?;
        let rows = stmt
            .query_map([], |row| {
                let kind: Option<String> = row.get(10)?;
                let brand: Option<String> = row.get(11)?;
                let model: Option<String> = row.get(12)?;
                let label = match (kind, brand, model) {
                    (None, _, _) => "Sem equipamento".to_string(),
                    (Some(k), Some(b), Some(m)) => equipment_label(&k, &b, &m),
                    _ => "Equipamento".to_string(),
                };
                Ok(Quote {
                    id: row.get(0)?,
                    client_id: row.get(1)?,
                    equipment_id: row.get(2)?,
                    description: row.get(3)?,
                    status: QuoteStatus::parse(&row.get::<_, String>(4)?),
                    labor_cents: row.get(5)?,
                    parts_cents: row.get(6)?,
                    discount_cents: row.get(7)?,
                    created_at: row.get(8)?,
                    client_name: row.get(9)?,
                    equipment_label: label,
                    paid_cents: row.get(13)?,
                })
            })
            .map_err(db_err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(db_err)
    }

    pub fn insert_quote(
        &self,
        client_id: i64,
        equipment_id: Option<i64>,
        description: &str,
        status: QuoteStatus,
        labor_cents: i64,
        parts_cents: i64,
        discount_cents: i64,
    ) -> DbResult<i64> {
        let now = today_iso();
        self.conn
            .execute(
                "INSERT INTO quotes
                    (client_id, equipment_id, description, status,
                     labor_cents, parts_cents, discount_cents, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
                params![
                    client_id,
                    equipment_id,
                    description,
                    status.as_str(),
                    labor_cents,
                    parts_cents,
                    discount_cents,
                    now
                ],
            )
            .map_err(db_err)?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_quote(
        &self,
        id: i64,
        client_id: i64,
        equipment_id: Option<i64>,
        description: &str,
        status: QuoteStatus,
        labor_cents: i64,
        parts_cents: i64,
        discount_cents: i64,
    ) -> DbResult<()> {
        self.conn
            .execute(
                "UPDATE quotes
                 SET client_id = ?2, equipment_id = ?3, description = ?4, status = ?5,
                     labor_cents = ?6, parts_cents = ?7, discount_cents = ?8, updated_at = ?9
                 WHERE id = ?1",
                params![
                    id,
                    client_id,
                    equipment_id,
                    description,
                    status.as_str(),
                    labor_cents,
                    parts_cents,
                    discount_cents,
                    today_iso()
                ],
            )
            .map_err(db_err)?;
        Ok(())
    }

    pub fn delete_quote(&self, id: i64) -> DbResult<()> {
        self.conn
            .execute("DELETE FROM quotes WHERE id = ?1", params![id])
            .map_err(db_err)?;
        Ok(())
    }

    // --------------------------------------------------------------- Pagamentos

    pub fn list_payments(&self) -> DbResult<Vec<Payment>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT p.id, p.quote_id, p.amount_cents, p.method, p.paid_on, p.notes,
                        c.name, q.description,
                        (q.labor_cents + q.parts_cents - q.discount_cents)
                 FROM payments p
                 JOIN quotes q ON q.id = p.quote_id
                 JOIN clients c ON c.id = q.client_id
                 ORDER BY p.paid_on DESC, p.id DESC",
            )
            .map_err(db_err)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Payment {
                    id: row.get(0)?,
                    quote_id: row.get(1)?,
                    amount_cents: row.get(2)?,
                    method: row.get(3)?,
                    paid_on: row.get(4)?,
                    notes: row.get(5)?,
                    client_name: row.get(6)?,
                    quote_description: row.get(7)?,
                    quote_total_cents: row.get(8)?,
                })
            })
            .map_err(db_err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(db_err)
    }

    pub fn insert_payment(
        &self,
        quote_id: i64,
        amount_cents: i64,
        method: &str,
        paid_on: &str,
        notes: &str,
    ) -> DbResult<i64> {
        self.conn
            .execute(
                "INSERT INTO payments (quote_id, amount_cents, method, paid_on, notes)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![quote_id, amount_cents, method, paid_on, notes],
            )
            .map_err(db_err)?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_payment(
        &self,
        id: i64,
        quote_id: i64,
        amount_cents: i64,
        method: &str,
        paid_on: &str,
        notes: &str,
    ) -> DbResult<()> {
        self.conn
            .execute(
                "UPDATE payments
                 SET quote_id = ?2, amount_cents = ?3, method = ?4, paid_on = ?5, notes = ?6
                 WHERE id = ?1",
                params![id, quote_id, amount_cents, method, paid_on, notes],
            )
            .map_err(db_err)?;
        Ok(())
    }

    pub fn delete_payment(&self, id: i64) -> DbResult<()> {
        self.conn
            .execute("DELETE FROM payments WHERE id = ?1", params![id])
            .map_err(db_err)?;
        Ok(())
    }

    // ------------------------------------------------------------ Dados de exemplo

    /// Popula o banco com dados de demonstração (apenas se estiver vazio).
    pub fn seed_demo(&self) -> DbResult<()> {
        if !self.is_empty() {
            return Ok(());
        }

        let ago = |days: i64| -> String {
            (Local::now() - Duration::days(days))
                .format("%Y-%m-%d")
                .to_string()
        };

        let maria = self.insert_client(
            "Maria Oliveira",
            "(11) 98765-4321",
            "maria.oliveira@email.com",
            "Rua das Acácias, 120 — São Paulo/SP",
            "Prefere contato por WhatsApp no período da tarde.",
        )?;
        let carlos = self.insert_client(
            "Carlos Pereira",
            "(11) 97654-3210",
            "carlos.pereira@email.com",
            "Av. Brasil, 45 — Guarulhos/SP",
            "",
        )?;
        let ana = self.insert_client(
            "Ana Souza",
            "(21) 96543-2109",
            "ana.souza@email.com",
            "Rua Bela Vista, 300 — Niterói/RJ",
            "Indicada pela cliente Maria Oliveira.",
        )?;

        let geladeira = self.insert_equipment(
            maria,
            "Geladeira",
            "Brastemp",
            "Frost Free 375L",
            "BRM54HK-0921",
            "Chegou com o cabo de força danificado.",
        )?;
        let lavadora = self.insert_equipment(
            maria,
            "Máquina de lavar",
            "Electrolux",
            "LAC11",
            "ELX-118842",
            "",
        )?;
        let micro = self.insert_equipment(
            carlos,
            "Micro-ondas",
            "Panasonic",
            "NN-ST65J",
            "PAN-553201",
            "Painel com display apagado.",
        )?;
        let ar = self.insert_equipment(
            ana,
            "Ar-condicionado",
            "LG",
            "Dual Inverter 12000 BTU",
            "LG-12QT-7741",
            "",
        )?;

        let q1 = self.insert_quote(
            maria,
            Some(geladeira),
            "Não está refrigerando. O compressor liga e desliga em poucos segundos. Verificar carga de gás e pressostato.",
            QuoteStatus::Aprovado,
            18000,
            32000,
            0,
        )?;
        let q2 = self.insert_quote(
            maria,
            Some(lavadora),
            "Troca do módulo de controle, substituição do filtro e teste do ciclo de centrifugação.",
            QuoteStatus::Concluido,
            15000,
            21000,
            1000,
        )?;
        let q3 = self.insert_quote(
            carlos,
            Some(micro),
            "Não aquece os alimentos. Suspeita de magnetron ou capacitor. Orçamento inclui peça e mão de obra.",
            QuoteStatus::Pendente,
            12000,
            28000,
            0,
        )?;
        let q4 = self.insert_quote(
            ana,
            Some(ar),
            "Instalação do suporte, limpeza completa e recarga de gás R410A.",
            QuoteStatus::Aprovado,
            25000,
            18000,
            0,
        )?;

        // Pagamentos parciais e integrais.
        self.insert_payment(q2, 35000, "Pix", &ago(9), "Pagamento integral na retirada.")?;
        self.insert_payment(q1, 20000, "Dinheiro", &ago(2), "Entrada de 40%.")?;
        self.insert_payment(
            q4,
            15000,
            "Pix",
            &ago(5),
            "Sinal para encomenda do suporte.",
        )?;

        let _ = q3;
        Ok(())
    }
}

/// Diretório de dados do usuário (`~/.local/share/oficina` no Linux).
fn data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("OFICINA_DATA_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("oficina");
        }
    }
    if cfg!(target_os = "windows") {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata).join("oficina");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("oficina");
    }
    PathBuf::from(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crud_flow() {
        let store = Store::in_memory().unwrap();
        let client = store
            .insert_client("Teste", "11999999999", "", "", "")
            .unwrap();
        let equip = store
            .insert_equipment(client, "Geladeira", "Brastemp", "X", "", "")
            .unwrap();
        let quote = store
            .insert_quote(
                client,
                Some(equip),
                "Teste",
                QuoteStatus::Aprovado,
                10000,
                5000,
                0,
            )
            .unwrap();
        store
            .insert_payment(quote, 5000, "Pix", &today_iso(), "")
            .unwrap();

        let quotes = store.list_quotes().unwrap();
        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0].total_cents(), 15000);
        assert_eq!(quotes[0].paid_cents, 5000);
        assert_eq!(quotes[0].balance_cents(), 10000);
        assert_eq!(quotes[0].client_name, "Teste");

        store.delete_client(client).unwrap();
        assert!(store.list_quotes().unwrap().is_empty());
        assert!(store.list_payments().unwrap().is_empty());
    }
}
