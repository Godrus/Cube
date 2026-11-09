//! Material palettes (CORE-12, contract C-10 MaterialMapping, PROVISIONAL: GOV-17 and RG-1 open).
//!
//! Separates two identities:
//!
//! * **content identity** ([`ContentId`], `<namespace>:<name>`): permanent, defined by the content
//!   that provides the material (the built-in package or a future mod); it is what a save must
//!   record;
//! * **runtime index** ([`MaterialIndex`]): a small number valid only inside one running
//!   palette; it is what the voxel channels store. It is *not* a permanent identity and must
//!   never be persisted without its mapping table.
//!
//! Core knows no content: this module has no built-in materials. Index `0` is air, reserved and
//! never assigned. Registration reports collisions of full identifiers; nothing is replaced
//! silently. A saved table can be resolved against the current palette strictly (an unknown
//! identifier is an error that lists them all) or in an explicit recovery mode, which registers
//! the unknown identifiers as **placeholders** that keep their original id, so no information is
//! lost and nothing is mapped to a different material.
//!
//! The table is plain data (`Vec<ContentId>`, index order); no binary format is defined here
//! (GOV-10 is open) and this contract is not Frozen.

use std::collections::HashMap;
use std::fmt;

use crate::material::MaterialIndex;

/// Longest namespace and name accepted.
pub const MAX_NAMESPACE_LEN: usize = 64;
/// Longest name part accepted.
pub const MAX_NAME_LEN: usize = 128;

/// Errors of the palette model.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PaletteError {
    /// The text is not `<namespace>:<name>` with the allowed characters and lengths.
    InvalidId(String),
    /// The full identifier is already registered (a collision); carries its existing index.
    Duplicate {
        /// The contested identifier.
        id: ContentId,
        /// The index it already has.
        existing: MaterialIndex,
    },
    /// All 65535 usable indices are taken.
    Full,
    /// A saved table names content this palette does not have (strict resolution); lists all.
    UnknownContent(Vec<ContentId>),
    /// A saved table is malformed (it lists the same identifier twice).
    InvalidTable(ContentId),
}

impl fmt::Display for PaletteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidId(s) => {
                write!(f, "invalid content id {s:?} (expected <namespace>:<name>)")
            }
            Self::Duplicate { id, .. } => write!(f, "content id {id} is already registered"),
            Self::Full => f.write_str("palette is full"),
            Self::UnknownContent(v) => {
                let names: Vec<String> = v.iter().map(ToString::to_string).collect();
                write!(f, "unknown content: {}", names.join(", "))
            }
            Self::InvalidTable(id) => write!(f, "table lists {id} twice"),
        }
    }
}

impl std::error::Error for PaletteError {}

/// Permanent content identity `<namespace>:<name>`.
///
/// The namespace part is `[a-z][a-z0-9_]*`. Name: non-empty, characters `[a-z0-9_./-]`. Both lowercase
/// ASCII so comparison is exact and platform independent.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContentId(String);

impl ContentId {
    /// Parses and validates `<namespace>:<name>`.
    pub fn parse(text: &str) -> Result<Self, PaletteError> {
        let bad = || PaletteError::InvalidId(text.to_owned());
        let (ns, name) = text.split_once(':').ok_or_else(bad)?;
        let ns_ok = !ns.is_empty()
            && ns.len() <= MAX_NAMESPACE_LEN
            && ns.starts_with(|c: char| c.is_ascii_lowercase())
            && ns
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        let name_ok = !name.is_empty()
            && name.len() <= MAX_NAME_LEN
            && name.chars().all(|c| {
                c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '.' | '/' | '-')
            });
        if ns_ok && name_ok {
            Ok(Self(text.to_owned()))
        } else {
            Err(bad())
        }
    }

    /// The identifier as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The namespace part.
    pub fn namespace(&self) -> &str {
        self.0.split_once(':').map_or("", |p| p.0)
    }
}

impl fmt::Display for ContentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Runtime palette: content identity <-> runtime index.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MaterialPalette {
    /// `entries[i]` is the content of index `i + 1`.
    entries: Vec<ContentId>,
    placeholder: Vec<bool>,
    index: HashMap<ContentId, u16>,
}

impl MaterialPalette {
    /// An empty palette (only air).
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of registered materials (air not counted).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `true` if nothing but air exists.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Registers `id` and returns its new runtime index. A duplicate full identifier is an
    /// error naming the existing index; indices are assigned in registration order, from 1.
    pub fn register(&mut self, id: ContentId) -> Result<MaterialIndex, PaletteError> {
        self.insert(id, false)
    }

    fn insert(&mut self, id: ContentId, placeholder: bool) -> Result<MaterialIndex, PaletteError> {
        if let Some(i) = self.index.get(&id) {
            return Err(PaletteError::Duplicate {
                id,
                existing: MaterialIndex::from_raw(*i),
            });
        }
        let next = self.entries.len() + 1;
        let raw = u16::try_from(next).map_err(|_| PaletteError::Full)?;
        self.index.insert(id.clone(), raw);
        self.entries.push(id);
        self.placeholder.push(placeholder);
        Ok(MaterialIndex::from_raw(raw))
    }

    /// The runtime index of `id`, if registered.
    pub fn index_of(&self, id: &ContentId) -> Option<MaterialIndex> {
        self.index.get(id).map(|i| MaterialIndex::from_raw(*i))
    }

    /// The content of a runtime index; `None` for air (`0`) and for unassigned indices.
    pub fn content_of(&self, m: MaterialIndex) -> Option<&ContentId> {
        usize::from(m.raw())
            .checked_sub(1)
            .and_then(|i| self.entries.get(i))
    }

    /// `true` if the index was registered as a recovery placeholder.
    pub fn is_placeholder(&self, m: MaterialIndex) -> bool {
        usize::from(m.raw())
            .checked_sub(1)
            .and_then(|i| self.placeholder.get(i))
            .copied()
            .unwrap_or(false)
    }

    /// The mapping table to save: content ids in runtime index order (entry `i` is index
    /// `i + 1`). Placeholders keep their original ids, so a table round-trips without loss.
    pub fn to_table(&self) -> Vec<ContentId> {
        self.entries.clone()
    }

    /// A palette from a saved table, preserving the indices (strict: duplicates are an error).
    pub fn from_table(table: &[ContentId]) -> Result<Self, PaletteError> {
        let mut p = Self::new();
        for id in table {
            p.register(id.clone()).map_err(|e| match e {
                PaletteError::Duplicate { id, .. } => PaletteError::InvalidTable(id),
                other => other,
            })?;
        }
        Ok(p)
    }

    /// Strict resolution of a saved table against this palette: every saved index must name
    /// content that exists here, else the error lists *all* unknown content.
    pub fn resolve_strict(&self, saved: &[ContentId]) -> Result<Mapping, PaletteError> {
        check_table(saved)?;
        let mut unknown = Vec::new();
        let mut map = Vec::with_capacity(saved.len());
        for id in saved {
            match self.index_of(id) {
                Some(m) => map.push(m),
                None => unknown.push(id.clone()),
            }
        }
        if unknown.is_empty() {
            Ok(Mapping { map })
        } else {
            Err(PaletteError::UnknownContent(unknown))
        }
    }

    /// Recovery-mode resolution: unknown content is registered here as a placeholder with its
    /// original id (visible through [`MaterialPalette::is_placeholder`]), never mapped to some
    /// other material. Returns the mapping and the placeholders that were created.
    pub fn resolve_recovery(
        &mut self,
        saved: &[ContentId],
    ) -> Result<(Mapping, Vec<ContentId>), PaletteError> {
        check_table(saved)?;
        let mut created = Vec::new();
        let mut map = Vec::with_capacity(saved.len());
        for id in saved {
            let m = match self.index_of(id) {
                Some(m) => m,
                None => {
                    created.push(id.clone());
                    self.insert(id.clone(), true)?
                }
            };
            map.push(m);
        }
        Ok((Mapping { map }, created))
    }
}

fn check_table(saved: &[ContentId]) -> Result<(), PaletteError> {
    let mut seen = std::collections::HashSet::new();
    for id in saved {
        if !seen.insert(id) {
            return Err(PaletteError::InvalidTable(id.clone()));
        }
    }
    Ok(())
}

/// Translation of saved runtime indices to the indices of the running palette.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mapping {
    map: Vec<MaterialIndex>,
}

impl Mapping {
    /// The running index for a saved index. Air stays air; an index the saved table does not
    /// have is `None` (corrupt data), never a guess.
    pub fn translate(&self, saved: MaterialIndex) -> Option<MaterialIndex> {
        match saved.raw() {
            0 => Some(MaterialIndex::from_raw(0)),
            n => self.map.get(usize::from(n) - 1).copied(),
        }
    }

    /// `true` if every saved index maps to itself (nothing needs rewriting).
    pub fn is_identity(&self) -> bool {
        self.map
            .iter()
            .enumerate()
            .all(|(i, m)| usize::from(m.raw()) == i + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(s: &str) -> ContentId {
        ContentId::parse(s).unwrap()
    }

    #[test]
    fn identifiers_are_validated_exactly() {
        for ok in ["pack_a:stone", "pack_c:blocks/oak-log.2", "a:b"] {
            assert_eq!(ContentId::parse(ok).unwrap().as_str(), ok);
        }
        assert_eq!(id("pack_a:stone").namespace(), "pack_a");
        for bad in [
            "",
            "stone",
            ":stone",
            "pack_a:",
            "Cube:stone",
            "pack_a:Stone",
            "1pack_a:x",
            "cu be:x",
            "pack_a:a b",
            "pack_a:a:b",
            "ünï:x",
        ] {
            assert!(
                matches!(ContentId::parse(bad), Err(PaletteError::InvalidId(_))),
                "{bad:?}"
            );
        }
        assert!(ContentId::parse(&format!("{}:x", "a".repeat(65))).is_err());
        assert!(ContentId::parse(&format!("a:{}", "x".repeat(129))).is_err());
        assert!(ContentId::parse(&format!("{}:{}", "a".repeat(64), "x".repeat(128))).is_ok());
    }

    #[test]
    fn indices_follow_registration_order_air_is_reserved_and_collisions_are_diagnosed() {
        let mut p = MaterialPalette::new();
        assert_eq!(p.register(id("pack_a:stone")).unwrap().raw(), 1);
        assert_eq!(p.register(id("pack_a:dirt")).unwrap().raw(), 2);
        assert_eq!(p.index_of(&id("pack_a:dirt")).map(|m| m.raw()), Some(2));
        assert_eq!(
            p.content_of(MaterialIndex::from_raw(1)),
            Some(&id("pack_a:stone"))
        );
        assert_eq!(
            p.content_of(MaterialIndex::from_raw(0)),
            None,
            "air has no content id"
        );
        assert_eq!(p.content_of(MaterialIndex::from_raw(9)), None);
        // The same short name in another namespace is a different material, not a collision.
        assert_eq!(p.register(id("pack_b:stone")).unwrap().raw(), 3);
        let err = p.register(id("pack_a:stone")).unwrap_err();
        assert!(matches!(&err, PaletteError::Duplicate { existing, .. } if existing.raw() == 1));
        assert_eq!(p.len(), 3, "a failed registration changes nothing");
    }

    #[test]
    fn the_palette_fills_up_at_65535_and_never_wraps() {
        let mut p = MaterialPalette::new();
        for i in 0..65535u32 {
            p.register(id(&format!("t:m{i}"))).unwrap();
        }
        assert_eq!(p.register(id("t:one-too-many")), Err(PaletteError::Full));
        assert_eq!(p.len(), 65535);
        assert_eq!(p.index_of(&id("t:m65534")).map(|m| m.raw()), Some(65535));
    }

    #[test]
    fn tables_round_trip_and_reject_duplicates() {
        let mut p = MaterialPalette::new();
        for n in ["pack_a:stone", "pack_a:dirt", "pack_b:glow"] {
            p.register(id(n)).unwrap();
        }
        let t = p.to_table();
        assert_eq!(MaterialPalette::from_table(&t).unwrap(), p);
        let dup = [id("a:x"), id("a:y"), id("a:x")];
        assert_eq!(
            MaterialPalette::from_table(&dup),
            Err(PaletteError::InvalidTable(id("a:x")))
        );
    }

    #[test]
    fn a_saved_table_maps_correctly_when_the_runtime_order_differs() {
        // The save was made with stone=1, dirt=2, glow=3; today's session registered them in
        // another order and has one extra material.
        let saved = [id("pack_a:stone"), id("pack_a:dirt"), id("pack_b:glow")];
        let mut now = MaterialPalette::new();
        for n in ["pack_b:glow", "pack_a:extra", "pack_a:dirt", "pack_a:stone"] {
            now.register(id(n)).unwrap();
        }
        let m = now.resolve_strict(&saved).unwrap();
        assert_eq!(m.translate(MaterialIndex::from_raw(1)).unwrap().raw(), 4);
        assert_eq!(m.translate(MaterialIndex::from_raw(2)).unwrap().raw(), 3);
        assert_eq!(m.translate(MaterialIndex::from_raw(3)).unwrap().raw(), 1);
        assert_eq!(
            m.translate(MaterialIndex::from_raw(0)).unwrap().raw(),
            0,
            "air stays air"
        );
        assert_eq!(
            m.translate(MaterialIndex::from_raw(4)),
            None,
            "an index the save never had"
        );
        assert!(!m.is_identity());
        // Reloading the same table gives the same mapping (stability).
        assert_eq!(now.resolve_strict(&saved).unwrap(), m);
        // A session with the same order needs no rewriting.
        let same = MaterialPalette::from_table(&saved).unwrap();
        assert!(same.resolve_strict(&saved).unwrap().is_identity());
    }

    #[test]
    fn unknown_content_is_an_error_in_strict_mode_and_a_visible_placeholder_in_recovery() {
        let saved = [
            id("pack_a:stone"),
            id("gone_pack:crystal"),
            id("gone_pack:ore"),
        ];
        let mut now = MaterialPalette::new();
        now.register(id("pack_a:stone")).unwrap();
        // Strict: nothing is substituted; every missing id is listed at once.
        assert_eq!(
            now.resolve_strict(&saved),
            Err(PaletteError::UnknownContent(vec![
                id("gone_pack:crystal"),
                id("gone_pack:ore")
            ]))
        );
        assert_eq!(now.len(), 1, "strict resolution changed nothing");
        // Recovery: placeholders keep the original ids, so the information survives a re-save.
        let (m, created) = now.resolve_recovery(&saved).unwrap();
        assert_eq!(created, vec![id("gone_pack:crystal"), id("gone_pack:ore")]);
        let crystal = m.translate(MaterialIndex::from_raw(2)).unwrap();
        assert!(now.is_placeholder(crystal) && !now.is_placeholder(MaterialIndex::from_raw(1)));
        assert_eq!(now.content_of(crystal), Some(&id("gone_pack:crystal")));
        assert_eq!(
            now.to_table(),
            vec![
                id("pack_a:stone"),
                id("gone_pack:crystal"),
                id("gone_pack:ore")
            ]
        );
        // Resolving again is stable and creates nothing more.
        let (m2, created2) = now.resolve_recovery(&saved).unwrap();
        assert_eq!((m2, created2), (m, vec![]));
        // When the mod returns, the same ids resolve strictly to the placeholder indices.
        assert!(now.resolve_strict(&saved).is_ok());
    }

    #[test]
    fn malformed_saved_tables_are_rejected_before_anything_is_registered() {
        let mut p = MaterialPalette::new();
        let dup = [id("a:x"), id("a:x")];
        assert!(p.resolve_recovery(&dup).is_err());
        assert!(p.is_empty());
        assert!(p.resolve_strict(&dup).is_err());
    }
}
