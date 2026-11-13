//! The character as one record, separate from the world (M1-A4, AI-15).
//!
//! Today the player is two client files next to the world: `player.state` (the pose) and
//! `game.state` (inventory, health, experience and the world-bound progress lists). A server
//! needs one engine-neutral record that it can store, send and validate without a client, and
//! that does not depend on how a world is stored. [`CharacterRecord`] is that record: the pose
//! and the game state under one version line, in a stable text form built from the two existing
//! text forms (so every value keeps its meaning and the files written today convert without
//! loss).
//!
//! ```text
//! cube-character 1
//! pose
//! cube-player 1
//! feet ...
//! game
//! cube-game 1
//! mode survival
//! ...
//! ```
//!
//! Strict on read: wrong header, a missing or repeated section or a damaged inner text is
//! refused, never repaired. The on-disk adoption (one file instead of two) is not done here: the
//! save format is experimental until GOV-10 (`docs/roadmap/WORK-PACKAGES.md`, M1-A4). The kill,
//! cache and ground lists inside the game state are world-bound progress of this prototype; their
//! split from the character is part of the multiplayer design (M8).

use crate::PlayerState;
use crate::game::GameState;
use crate::port::{decode_player, encode_player};

/// Header line of the text form (the version is the number).
pub const HEADER: &str = "cube-character 1";

/// Everything a save keeps of one character.
#[derive(Debug, Clone, PartialEq)]
pub struct CharacterRecord {
    /// Where the character stands and looks.
    pub pose: PlayerState,
    /// Items, mode, objective, health, experience and the world-bound progress lists.
    pub game: GameState,
}

impl CharacterRecord {
    /// The stable text form.
    pub fn encode(&self) -> String {
        format!(
            "{HEADER}\npose\n{}game\n{}",
            encode_player(&self.pose),
            self.game.encode()
        )
    }

    /// Reads [`CharacterRecord::encode`] output; `None` for anything that is not a complete,
    /// valid record.
    pub fn decode(text: &str) -> Option<Self> {
        let mut lines = text.lines();
        if lines.next()? != HEADER || lines.next()? != "pose" {
            return None;
        }
        let (mut pose, mut game) = (String::new(), String::new());
        let mut in_game = false;
        for l in lines {
            if l == "game" {
                if in_game {
                    return None;
                }
                in_game = true;
                continue;
            }
            if l == "pose" {
                return None;
            }
            let target = if in_game { &mut game } else { &mut pose };
            target.push_str(l);
            target.push('\n');
        }
        if !in_game {
            return None;
        }
        Some(Self {
            pose: decode_player(&pose)?,
            game: GameState::decode(&game)?,
        })
    }

    /// The record of a character that today lives in the two client files: their texts as they
    /// are on disk. `None` if either is damaged.
    pub fn from_files(player_text: &str, game_text: &str) -> Option<Self> {
        Some(Self {
            pose: decode_player(player_text)?,
            game: GameState::decode(game_text)?,
        })
    }

    /// The two file texts of this record (the inverse of [`CharacterRecord::from_files`]).
    pub fn to_files(&self) -> (String, String) {
        (encode_player(&self.pose), self.game.encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::ItemId;

    fn record() -> CharacterRecord {
        let mut game = GameState::default();
        game.inventory.add(ItemId(5), 7);
        game.progress = crate::progression::Progress::from_xp(37).unwrap();
        game.killed.insert((3, -4));
        game.drop_to_ground(ItemId(6), 2, [1, 2, 3]);
        CharacterRecord {
            pose: PlayerState {
                feet: [1.5, -10.0, 33.25],
                yaw: 0.5,
                pitch: -0.25,
                walking: true,
                third_person: false,
            },
            game,
        }
    }

    #[test]
    fn the_record_round_trips_and_its_form_is_stable() {
        let r = record();
        let text = r.encode();
        assert_eq!(CharacterRecord::decode(&text), Some(r.clone()));
        assert_eq!(
            CharacterRecord::decode(&text).unwrap().encode(),
            text,
            "encoding is canonical"
        );
        // The head of the text is pinned: a change of the form is a new version, not a silent edit.
        assert!(
            text.starts_with(
                "cube-character 1\npose\ncube-player 1\nfeet 1.5 -10 33.25\nview 0.5 -0.25\nwalking 1\nthird_person 0\ngame\ncube-game 1\nmode survival\n"
            ),
            "{text}"
        );
        assert!(text.contains("ground 6 2 1 2 3\n") && text.contains("xp 37\n"));
    }

    #[test]
    fn the_two_files_of_today_convert_without_loss_in_both_directions() {
        let r = record();
        let (player, game) = r.to_files();
        assert_eq!(CharacterRecord::from_files(&player, &game), Some(r.clone()));
        // A pair written before GP-01 and before experience (no xp or ground lines) converts.
        let old_game = GameState::default().encode();
        let old = CharacterRecord::from_files(&player, &old_game).unwrap();
        assert!(old.game.ground.is_empty() && old.game.progress.xp() == 0);
        assert_eq!(CharacterRecord::decode(&old.encode()), Some(old));
        // Damage in either file is refused.
        assert!(CharacterRecord::from_files("cube-player 1\nfeet x\n", &game).is_none());
        assert!(CharacterRecord::from_files(&player, "cube-game 1\nmode x\n").is_none());
    }

    #[test]
    fn a_damaged_or_incomplete_record_is_refused_never_repaired() {
        let good = record().encode();
        assert!(CharacterRecord::decode("").is_none());
        assert!(
            CharacterRecord::decode(&good.replace("cube-character 1", "cube-character 2"))
                .is_none()
        );
        // A missing section, a repeated section, sections in the wrong order, trailing junk.
        assert!(CharacterRecord::decode(&good.replace("\ngame\n", "\n")).is_none());
        assert!(CharacterRecord::decode(&good.replace("\ngame\n", "\ngame\n\ngame\n")).is_none());
        assert!(CharacterRecord::decode(&good.replace("pose\n", "game\n")).is_none());
        assert!(CharacterRecord::decode(&format!("{good}junk 1\n")).is_none());
        // A damaged inner text.
        assert!(CharacterRecord::decode(&good.replace("feet 1.5", "feet nan")).is_none());
        assert!(CharacterRecord::decode(&good.replace("xp 37", "xp 99999")).is_none());
    }
}
