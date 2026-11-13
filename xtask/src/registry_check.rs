//! Decision registry checks for `cargo xtask docscheck` (DOC-05).
//!
//! * every decision id (`DEC-n`, `AOD-n`, `OAM-n`) named in a tracked document outside the
//!   archive exists in the registry, as an entry id or as an alias of an entry;
//! * a `CONFIRMED` entry names its confirmation evidence;
//! * an id is never reused: no entry heading appears twice and no alias is a `DEC-n` id (an
//!   alias of that form would be a second claim on an id);
//! * a contract row of `docs/api-contracts/` whose maturity cell is `Frozen` cites a `DEC-n`
//!   entry that is `CONFIRMED` (a Frozen contract must be Owner-Approved, plan 3.1).

use std::collections::{BTreeMap, BTreeSet};

/// The registry entry fields these checks need.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Entry {
    pub id: String,
    pub aliases: Vec<String>,
    pub status: String,
    pub evidence: String,
}

/// Decision ids named in `text`, in order of appearance: `DEC-`, `AOD-` or `OAM-` followed by
/// digits, not preceded by a letter or digit.
pub fn decision_ids(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 4 < bytes.len() {
        // Bytes, not str slices: the text is UTF-8 and index i may fall inside a letter.
        let tag = &bytes[i..i + 4];
        let known = tag == b"DEC-" || tag == b"AOD-" || tag == b"OAM-";
        let before_ok = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
        if known && before_ok {
            let digits = bytes[i + 4..]
                .iter()
                .take_while(|b| b.is_ascii_digit())
                .count();
            if digits > 0 {
                out.push(text[i..i + 4 + digits].to_string());
                i += 4 + digits;
                continue;
            }
        }
        i += 1;
    }
    out
}

/// Entries of the registry text (`## DEC-n  title` headings, `key: value` lines below).
pub fn parse_registry(text: &str) -> Vec<Entry> {
    let mut entries: Vec<Entry> = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("## DEC-") {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            entries.push(Entry {
                id: format!("DEC-{digits}"),
                ..Entry::default()
            });
            continue;
        }
        let Some(e) = entries.last_mut() else {
            continue;
        };
        if let Some(v) = line.strip_prefix("aliases:") {
            e.aliases.extend(decision_ids(v));
        } else if let Some(v) = line.strip_prefix("status:") {
            e.status = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("confirmation_evidence:") {
            e.evidence = v.trim().to_string();
        }
    }
    entries
}

/// Problems inside the registry itself.
pub fn check_entries(entries: &[Entry]) -> Vec<String> {
    let mut bad = Vec::new();
    let mut seen = BTreeSet::new();
    for e in entries {
        if !seen.insert(e.id.clone()) {
            bad.push(format!("decision registry: {} appears twice", e.id));
        }
        if e.status == "CONFIRMED" {
            let ev = e.evidence.trim();
            if ev.is_empty() || ev.to_uppercase().starts_with("NONE") {
                bad.push(format!(
                    "decision registry: {} is CONFIRMED without confirmation_evidence",
                    e.id
                ));
            }
        }
        for a in &e.aliases {
            if a.starts_with("DEC-") && a != &e.id {
                bad.push(format!(
                    "decision registry: {} lists {a} as an alias (an id is never reused)",
                    e.id
                ));
            }
        }
    }
    bad
}

/// Ids (entry ids and aliases) the registry knows.
pub fn known_ids(entries: &[Entry]) -> BTreeSet<String> {
    let mut k = BTreeSet::new();
    for e in entries {
        k.insert(e.id.clone());
        k.extend(e.aliases.iter().cloned());
    }
    k
}

/// Decision ids named in `text` that the registry does not know, each once.
pub fn unknown_ids(text: &str, known: &BTreeSet<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    decision_ids(text)
        .into_iter()
        .filter(|id| !known.contains(id) && seen.insert(id.clone()))
        .collect()
}

/// Contract rows (`| ... |` lines) whose maturity cell is exactly `Frozen` but which do not cite
/// a `CONFIRMED` `DEC-n` entry.
pub fn frozen_without_approval(text: &str, status_of: &BTreeMap<String, String>) -> Vec<String> {
    text.lines()
        .filter(|l| l.trim_start().starts_with('|'))
        .filter(|l| l.split('|').any(|c| c.trim().trim_matches('*') == "Frozen"))
        .filter(|l| {
            !decision_ids(l).iter().any(|id| {
                id.starts_with("DEC-") && status_of.get(id).is_some_and(|s| s == "CONFIRMED")
            })
        })
        .map(|l| l.trim().chars().take(80).collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const REG: &str = "\
# Registry

## DEC-001  First
id: DEC-001
aliases: [ODM-12 (partly), AOD-01]
status: CONFIRMED
confirmation_evidence: E-CHAT

## DEC-002  Second
id: DEC-002
aliases: [OAM-02]
status: PROPOSED
";

    #[test]
    fn ids_are_found_with_boundaries() {
        assert_eq!(
            decision_ids("see DEC-004..DEC-027, AOD-01 and (OAM-12); XDEC-5, DEC-, DEC-x"),
            vec!["DEC-004", "DEC-027", "AOD-01", "OAM-12"]
        );
        assert!(decision_ids("nothing here").is_empty());
        // Non-ASCII text around ids (the documents are partly Russian).
        assert_eq!(
            decision_ids("решение DEC-012 принято; см. АОД и OAM-3."),
            vec!["DEC-012", "OAM-3"]
        );
    }

    #[test]
    fn registry_is_parsed_and_aliases_count_as_known() {
        let e = parse_registry(REG);
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].aliases, vec!["AOD-01"]);
        let k = known_ids(&e);
        assert!(k.contains("DEC-002") && k.contains("OAM-02") && k.contains("AOD-01"));
        assert_eq!(
            unknown_ids("DEC-001 AOD-01 OAM-05 OAM-05 DEC-099", &k),
            vec!["OAM-05", "DEC-099"]
        );
    }

    #[test]
    fn a_confirmed_entry_needs_evidence_and_ids_are_not_reused() {
        assert!(check_entries(&parse_registry(REG)).is_empty());
        let bad = format!(
            "{REG}\n## DEC-003  Third\nid: DEC-003\nstatus: CONFIRMED\nconfirmation_evidence: NONE (agent)\n\n## DEC-001  Again\nstatus: PROPOSED\n\n## DEC-004  Fourth\naliases: [DEC-002]\nstatus: PROPOSED\n"
        );
        let problems = check_entries(&parse_registry(&bad));
        assert_eq!(problems.len(), 3, "{problems:?}");
        assert!(problems.iter().any(|p| p.contains("DEC-003 is CONFIRMED")));
        assert!(problems.iter().any(|p| p.contains("DEC-001 appears twice")));
        assert!(problems.iter().any(|p| p.contains("DEC-004 lists DEC-002")));
    }

    #[test]
    fn a_frozen_contract_row_must_cite_a_confirmed_decision() {
        let mut st = BTreeMap::new();
        st.insert("DEC-010".to_string(), "CONFIRMED".to_string());
        st.insert("DEC-011".to_string(), "PROPOSED".to_string());
        let text = "\
| C-01 | Frozen | DEC-010 |
| C-02 | Frozen | DEC-011 |
| C-03 | Frozen | none |
| C-04 | Provisional | DEC-011 |
Frozen is a status in prose, not a row.";
        let bad = frozen_without_approval(text, &st);
        assert_eq!(bad.len(), 2, "{bad:?}");
        assert!(bad[0].contains("C-02") && bad[1].contains("C-03"));
    }
}
