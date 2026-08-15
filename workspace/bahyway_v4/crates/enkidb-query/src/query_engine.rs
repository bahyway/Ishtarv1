//! EnkiQuery — HeptaScript W5H2 query execution over EnkiDb.
//!
//! Bridges the EnkiDb journal with the HeptaScript executor. All candidates
//! are sourced from the in-memory journal; the W5H2 engine applies WHO/WHERE/
//! WHEN/WHY/WHAT/HOW/HOW_MUCH filtering natively against decoded EAV triples.

use bahyway_core::BahywayError;
use enkidb_engine::EnkiDb;
use heptascript::{parse_query, execute, QueryResult};

/// Execute a HeptaScript W5H2 source string against an `EnkiDb`.
///
/// Candidates are drawn from all entities known to the journal.
/// Returns an error only on parse failure; an empty `QueryResult` is normal.
pub fn query(db: &EnkiDb, hepta_source: &str) -> Result<QueryResult, BahywayError> {
    let q = parse_query(hepta_source)
        .map_err(|e| BahywayError::Internal(e.to_string()))?;
    Ok(execute(&q, db.journal()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;
    use enkidb_engine::EnkiDb;
    use enkidb_journal::entry::EavTriple;
    use enkidb_kaki::{EventKaki, KakiRole, mint::KakiMinter};
    use akkvalue::{AkkValue, codec::encode as codec_encode};
    use bahyway_crc::crc16;

    fn attr_hash(name: &str) -> u32 {
        crc16(name.as_bytes()) as u32
    }

    fn setup(tribe: u16) -> (KakiMinter, EnkiDb) {
        let tid = TribeId::from_u16(tribe);
        (KakiMinter::new(tid), EnkiDb::new(tid))
    }

    fn push(db: &mut EnkiDb, m: &KakiMinter, attrs: &[(&str, AkkValue)]) -> enkidb_kaki::IdentityKaki {
        let entity = enkidb_kaki::IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let event  = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
        let eav: Vec<EavTriple> = attrs.iter()
            .map(|(name, val)| EavTriple::new(attr_hash(name), codec_encode(val)))
            .collect();
        db.register_particle(&entity).unwrap();
        db.append_event(event, entity.clone(), 1, eav).unwrap();
        entity
    }

    #[test]
    fn query_string_eq() {
        let (m, mut db) = setup(0x0001);
        push(&mut db, &m, &[("city.name", AkkValue::Text("Najaf".into()))]);
        push(&mut db, &m, &[("city.name", AkkValue::Text("Baghdad".into()))]);

        let res = query(&db, "WHO T.E\nWHERE E[city.name] = \"Najaf\"").unwrap();
        assert_eq!(res.matched.len(), 1);
        assert_eq!(res.evaluated, 2);
    }

    #[test]
    fn query_int_lt() {
        let (m, mut db) = setup(0x0002);
        push(&mut db, &m, &[("score", AkkValue::Int(80))]);
        push(&mut db, &m, &[("score", AkkValue::Int(95))]);

        let res = query(&db, "WHO T.E\nWHERE E[score] >= 90").unwrap();
        assert_eq!(res.matched.len(), 1);
    }

    #[test]
    fn query_no_conditions_returns_all() {
        let (m, mut db) = setup(0x0003);
        for _ in 0..3 {
            push(&mut db, &m, &[("x", AkkValue::Int(1))]);
        }
        let res = query(&db, "WHO T.E").unwrap();
        assert_eq!(res.matched.len(), 3);
    }

    #[test]
    fn query_syntax_error_returns_err() {
        let (_, db) = setup(0x0004);
        assert!(query(&db, "INVALID syntax !!").is_err());
    }

    #[test]
    fn query_with_how_much_limit() {
        let (m, mut db) = setup(0x0005);
        for i in 0..5i64 {
            push(&mut db, &m, &[("idx", AkkValue::Int(i))]);
        }
        let res = query(&db, "WHO T.E\nHOW_MUCH LIMIT 3").unwrap();
        assert_eq!(res.matched.len(), 3);
    }

    #[test]
    fn query_exists_condition() {
        let (m, mut db) = setup(0x0006);
        push(&mut db, &m, &[("alert.flag", AkkValue::Bool(true))]);
        push(&mut db, &m, &[("status",    AkkValue::Text("ok".into()))]);

        let res = query(&db, "WHO T.E\nWHERE E[alert.flag] EXISTS").unwrap();
        assert_eq!(res.matched.len(), 1);
    }
}
