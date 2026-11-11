//! First gameplay state of the local player: inventory with a hotbar, creative or survival rules,
//! and one small objective. **Prototype rules (PROPOSED, not an approved design)**; engine-neutral,
//! no authority over the world: every block change still goes through the host's Editing API, and
//! this state only follows the result.
//!
//! Rules of the prototype:
//! * items are block items; the item id is the material id (1..=5);
//! * survival: removing a block puts one item of its material into the inventory (refused when it
//!   would not fit), placing one needs and consumes one item of the selected slot;
//! * creative: the hotbar offers every material without limit and nothing is consumed;
//! * order of effects: the world change is committed first, the inventory follows only after a
//!   confirmed commit, so a refused or failed edit never costs or creates an item. The journal and
//!   the game file are two stores and cannot be one atomic action, so they are **reconciled**
//!   instead: before the world is changed, the game file records a write-ahead intent (the
//!   operation sequence and the effect); the effect is applied and the intent cleared in one file
//!   write after the commit; after a restart an intent that is still there is settled against the
//!   journal (confirmed: apply the effect once; not confirmed: drop it). See
//!   [`GameState::reconcile`]. Limits: the game file is written with a flushed temporary file and
//!   a rename, with no directory sync and no device durability claim (DG-2).

/// Slots in the hotbar (the first slots of the inventory).
pub const HOTBAR: usize = 9;
/// Slots in the whole inventory.
pub const SLOTS: usize = 27;
/// Largest stack.
pub const MAX_STACK: u32 = 64;

/// An item kind. Block items use the material id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemId(pub u16);

/// What an item is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemDefinition {
    /// The item.
    pub id: ItemId,
    /// Display name.
    pub name: &'static str,
    /// Largest stack.
    pub max_stack: u32,
    /// Material placed when used.
    pub places: Option<u16>,
}

/// How many of the items are blocks (the first ones): creative mode offers exactly these.
pub const BLOCK_ITEMS: usize = 5;

/// The `i`-th placeable item (what creative mode offers in hotbar slot `i`).
pub fn placeable(i: usize) -> Option<&'static ItemDefinition> {
    ITEMS.iter().filter(|d| d.places.is_some()).nth(i)
}

/// All items of the prototype.
pub const ITEMS: [ItemDefinition; 13] = [
    ItemDefinition {
        id: ItemId(1),
        name: "Stone",
        max_stack: MAX_STACK,
        places: Some(1),
    },
    ItemDefinition {
        id: ItemId(2),
        name: "Dirt",
        max_stack: MAX_STACK,
        places: Some(2),
    },
    ItemDefinition {
        id: ItemId(3),
        name: "Grass",
        max_stack: MAX_STACK,
        places: Some(3),
    },
    ItemDefinition {
        id: ItemId(4),
        name: "Sand",
        max_stack: MAX_STACK,
        places: Some(4),
    },
    ItemDefinition {
        id: ItemId(5),
        name: "Red brick",
        max_stack: MAX_STACK,
        places: Some(5),
    },
    ItemDefinition {
        id: ItemId(6),
        name: "Slime gel",
        max_stack: MAX_STACK,
        places: None,
    },
    ItemDefinition {
        id: ItemId(7),
        name: "Snow",
        max_stack: MAX_STACK,
        places: Some(7),
    },
    ItemDefinition {
        id: ItemId(8),
        name: "Wood",
        max_stack: MAX_STACK,
        places: Some(8),
    },
    ItemDefinition {
        id: ItemId(9),
        name: "Leaves",
        max_stack: MAX_STACK,
        places: Some(9),
    },
    ItemDefinition {
        id: ItemId(10),
        name: "Moss",
        max_stack: MAX_STACK,
        places: Some(10),
    },
    ItemDefinition {
        id: ItemId(11),
        name: "Coal ore",
        max_stack: MAX_STACK,
        places: Some(11),
    },
    ItemDefinition {
        id: ItemId(12),
        name: "Iron ore",
        max_stack: MAX_STACK,
        places: Some(12),
    },
    ItemDefinition {
        id: IRON_SWORD_ITEM,
        name: "Iron sword",
        max_stack: 1,
        places: None,
    },
];

/// The iron sword (a weapon: the selected hotbar item decides which weapon a swing uses).
pub const IRON_SWORD_ITEM: ItemId = ItemId(13);

/// The weapon a swing uses when `item` is the selected hotbar item (anything else: the wooden
/// sword the player always has).
pub fn weapon_of(item: Option<ItemId>) -> &'static crate::combat::WeaponDef {
    if item == Some(IRON_SWORD_ITEM) {
        &crate::combat::IRON_SWORD
    } else {
        &crate::combat::WOODEN_SWORD
    }
}

/// A crafting recipe (data): inputs consumed, one output made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Recipe {
    /// Display name.
    pub name: &'static str,
    /// Items and counts consumed.
    pub inputs: &'static [(ItemId, u32)],
    /// Item and count made.
    pub output: (ItemId, u32),
}

/// The recipes of the prototype (no stations, no skill): prototype values, not a balance.
pub const RECIPES: [Recipe; 1] = [Recipe {
    name: "Iron sword",
    inputs: &[(ItemId(12), 3), (ItemId(8), 2)],
    output: (IRON_SWORD_ITEM, 1),
}];

/// Why a craft did not happen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CraftRefused {
    /// Creative mode has no use for crafting (it creates and consumes nothing).
    Creative,
    /// No such recipe.
    Unknown,
    /// Not enough of an input.
    Missing,
    /// The result would not fit into the inventory.
    NoRoom,
}

/// The definition of an item, if it exists.
pub fn definition(id: ItemId) -> Option<ItemDefinition> {
    ITEMS.iter().find(|d| d.id == id).copied()
}

/// A number of items of one kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemStack {
    /// The kind.
    pub item: ItemId,
    /// How many (1..=max stack).
    pub count: u32,
}

/// The player's items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inventory {
    slots: [Option<ItemStack>; SLOTS],
    selected: usize,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            slots: [None; SLOTS],
            selected: 0,
        }
    }
}

impl Inventory {
    /// An empty inventory.
    pub fn new() -> Self {
        Self::default()
    }

    /// The slots.
    pub fn slots(&self) -> &[Option<ItemStack>; SLOTS] {
        &self.slots
    }

    /// The selected hotbar slot.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Selects a hotbar slot (ignored outside the hotbar).
    pub fn select(&mut self, slot: usize) {
        if slot < HOTBAR {
            self.selected = slot;
        }
    }

    /// Moves the selection by `delta` slots, wrapping around the hotbar.
    pub fn scroll(&mut self, delta: i32) {
        self.selected = (self.selected as i32 + delta).rem_euclid(HOTBAR as i32) as usize;
    }

    /// The item of the selected slot.
    pub fn selected_item(&self) -> Option<ItemId> {
        self.slots[self.selected].map(|s| s.item)
    }

    /// How many items of a kind there are in total.
    pub fn count_of(&self, item: ItemId) -> u32 {
        self.slots
            .iter()
            .flatten()
            .filter(|s| s.item == item)
            .map(|s| s.count)
            .sum()
    }

    fn room_for(&self, item: ItemId) -> u32 {
        let Some(def) = definition(item) else {
            return 0;
        };
        self.slots
            .iter()
            .map(|s| match s {
                Some(st) if st.item == item => def.max_stack - st.count,
                Some(_) => 0,
                None => def.max_stack,
            })
            .sum()
    }

    /// `true` if `count` items of the kind fit.
    pub fn can_add(&self, item: ItemId, count: u32) -> bool {
        self.room_for(item) >= count
    }

    /// Adds items, filling existing stacks first and then empty slots (hotbar first). Returns how
    /// many did not fit (an unknown item never fits).
    pub fn add(&mut self, item: ItemId, mut count: u32) -> u32 {
        let Some(def) = definition(item) else {
            return count;
        };
        for s in self.slots.iter_mut().flatten() {
            if s.item == item && count > 0 {
                let put = (def.max_stack - s.count).min(count);
                s.count += put;
                count -= put;
            }
        }
        for slot in self.slots.iter_mut() {
            if count == 0 {
                break;
            }
            if slot.is_none() {
                let put = def.max_stack.min(count);
                *slot = Some(ItemStack { item, count: put });
                count -= put;
            }
        }
        count
    }

    /// Removes `count` items of a kind (across stacks); `false` and nothing changed if there are
    /// fewer.
    pub fn remove(&mut self, item: ItemId, count: u32) -> bool {
        if self.count_of(item) < count {
            return false;
        }
        let mut left = count;
        for slot in self.slots.iter_mut().rev() {
            if left == 0 {
                break;
            }
            if let Some(st) = slot
                && st.item == item
            {
                let take = st.count.min(left);
                st.count -= take;
                left -= take;
                if st.count == 0 {
                    *slot = None;
                }
            }
        }
        true
    }

    /// Takes one item out of the selected slot, if it holds any.
    pub fn take_selected(&mut self) -> Option<ItemId> {
        let s = self.slots[self.selected].as_mut()?;
        let item = s.item;
        s.count -= 1;
        if s.count == 0 {
            self.slots[self.selected] = None;
        }
        Some(item)
    }

    /// Swaps two slots (a drag in the inventory panel); stacks of the same kind are merged.
    pub fn swap(&mut self, a: usize, b: usize) {
        if a >= SLOTS || b >= SLOTS || a == b {
            return;
        }
        if let (Some(x), Some(y)) = (self.slots[a], self.slots[b])
            && x.item == y.item
            && let Some(def) = definition(x.item)
        {
            let put = (def.max_stack - y.count).min(x.count);
            self.slots[b] = Some(ItemStack {
                item: y.item,
                count: y.count + put,
            });
            self.slots[a] = (x.count > put).then_some(ItemStack {
                item: x.item,
                count: x.count - put,
            });
            return;
        }
        self.slots.swap(a, b);
    }
}

/// Which rules apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Every material is available without limit.
    Creative,
    /// Blocks come from mining and are used up by building.
    Survival,
}

/// The first objective: mine some blocks and build with them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Objective {
    /// Blocks to mine.
    pub mine_goal: u32,
    /// Blocks to place.
    pub place_goal: u32,
    /// Blocks mined so far.
    pub mined: u32,
    /// Blocks placed so far.
    pub placed: u32,
}

impl Objective {
    /// The shelter objective of the prototype.
    pub const SHELTER: Self = Self {
        mine_goal: 8,
        place_goal: 12,
        mined: 0,
        placed: 0,
    };

    /// Identifier of the objective (for saving).
    pub const ID: &'static str = "shelter-1";

    /// Title and rule text.
    pub fn text(&self) -> (&'static str, String) {
        (
            "Build a shelter",
            format!(
                "Mine {} blocks ({}/{}) and place {} ({}/{})",
                self.mine_goal,
                self.mined.min(self.mine_goal),
                self.mine_goal,
                self.place_goal,
                self.placed.min(self.place_goal),
                self.place_goal
            ),
        )
    }

    /// Done?
    pub fn done(&self) -> bool {
        self.mined >= self.mine_goal && self.placed >= self.place_goal
    }
}

/// What a world edit that is under way will do to the game state once the world confirms it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingKind {
    /// A block of this material is being mined: one item is added on confirmation.
    Mined {
        /// The material of the block.
        material: u16,
    },
    /// A block is being placed from the selected slot: one item is used on confirmation.
    Placed,
}

/// The write-ahead intent of one world edit: recorded in the game file **before** the world is
/// changed and cleared in the same file write that applies its effect, so the two stores can be
/// reconciled after a crash at any point (see [`GameState::reconcile`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingEdit {
    /// The sequence of the operation id the edit is submitted under.
    pub seq: u64,
    /// The effect to apply if the world confirmed it.
    pub kind: PendingKind,
}

/// What [`GameState::reconcile`] found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reconciled {
    /// No edit was under way.
    Nothing,
    /// The world had the change and the game state lacked its effect: the effect was applied.
    Applied,
    /// The world never got the change: the intent was dropped, the game state is unchanged.
    Dropped,
}

/// Everything a save keeps of the player's game state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    /// Items.
    pub inventory: Inventory,
    /// Rules.
    pub mode: Mode,
    /// The objective.
    pub objective: Objective,
    /// An edit that was started and not yet finished (normally `None`).
    pub pending: Option<PendingEdit>,
    /// The player's health (prototype maximum, see `combat`). Files without a `health` line
    /// (written before combat existed) load with full health.
    pub health: crate::combat::Health,
    /// Columns whose creature the player killed (it does not come back, and its loot was given
    /// in the same file write, so a kill and its loot are saved together or not at all).
    pub killed: std::collections::BTreeSet<(i64, i64)>,
    /// Experience (the level is derived from it). Files without an `xp` line load with none.
    pub progress: crate::progression::Progress,
    /// Altars (ruin centre columns) whose cache the player took; it is never refilled.
    pub looted: std::collections::BTreeSet<(i64, i64)>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            inventory: Inventory::new(),
            mode: Mode::Survival,
            objective: Objective::SHELTER,
            pending: None,
            health: crate::combat::Health::full(crate::combat::PLAYER_MAX_HEALTH),
            killed: std::collections::BTreeSet::new(),
            progress: crate::progression::Progress::new(),
            looted: std::collections::BTreeSet::new(),
        }
    }
}

/// What a mining or placing request turned out to be, for the player's message line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// It happened.
    Done,
    /// Nothing there to act on.
    Nothing,
    /// The rules forbid it; the text says why.
    Refused(String),
}

impl GameState {
    /// The material the selected slot would place: creative offers materials 1..=5 on the hotbar.
    pub fn selected_material(&self) -> Option<u16> {
        match self.mode {
            Mode::Creative => {
                let i = self.inventory.selected();
                placeable(i).and_then(|d| d.places)
            }
            Mode::Survival => self
                .inventory
                .selected_item()
                .and_then(definition)
                .and_then(|d| d.places),
        }
    }

    /// Can a block of `material` that was just mined be kept? (Checked before the world changes.)
    pub fn can_keep(&self, material: u16) -> bool {
        self.mode == Mode::Creative || self.inventory.can_add(ItemId(material), 1)
    }

    /// A block was removed from the world (the commit succeeded).
    pub fn on_mined(&mut self, material: u16) -> Option<String> {
        if self.mode == Mode::Survival {
            let left = self.inventory.add(ItemId(material), 1);
            debug_assert_eq!(left, 0, "can_keep was checked first");
        }
        let was = self.objective.done();
        self.objective.mined += 1;
        (!was && self.objective.done()).then(|| "Objective complete: shelter built!".to_owned())
    }

    /// A block was placed in the world (the commit succeeded).
    pub fn on_placed(&mut self) -> Option<String> {
        if self.mode == Mode::Survival {
            let taken = self.inventory.take_selected();
            debug_assert!(taken.is_some(), "the selected slot was checked first");
        }
        let was = self.objective.done();
        self.objective.placed += 1;
        (!was && self.objective.done()).then(|| "Objective complete: shelter built!".to_owned())
    }

    /// Crafts recipe `i`: all inputs are consumed and the output added, or nothing changes.
    pub fn craft(&mut self, i: usize) -> Result<(), CraftRefused> {
        if self.mode == Mode::Creative {
            return Err(CraftRefused::Creative);
        }
        let r = RECIPES.get(i).ok_or(CraftRefused::Unknown)?;
        if r.inputs
            .iter()
            .any(|&(it, n)| self.inventory.count_of(it) < n)
        {
            return Err(CraftRefused::Missing);
        }
        // Try on a copy: the output must fit once the inputs are gone.
        let mut trial = self.inventory.clone();
        for &(it, n) in r.inputs {
            trial.remove(it, n);
        }
        if trial.add(r.output.0, r.output.1) > 0 {
            return Err(CraftRefused::NoRoom);
        }
        self.inventory = trial;
        Ok(())
    }

    /// Puts loot into the inventory (survival only: creative creates and consumes nothing).
    /// Returns how many items did not fit; they are **lost** in this prototype (there are no item
    /// entities on the ground yet), and the caller must say so to the player.
    pub fn grant_loot(&mut self, item: ItemId, count: u32) -> u32 {
        if self.mode == Mode::Creative {
            return 0;
        }
        self.inventory.add(item, count)
    }

    /// Records that a block of `material` is about to be mined under operation `seq`. Save the
    /// state before the world is changed.
    pub fn begin_mined(&mut self, seq: u64, material: u16) {
        self.pending = Some(PendingEdit {
            seq,
            kind: PendingKind::Mined { material },
        });
    }

    /// Records that a block is about to be placed under operation `seq`. Save the state before
    /// the world is changed.
    pub fn begin_placed(&mut self, seq: u64) {
        self.pending = Some(PendingEdit {
            seq,
            kind: PendingKind::Placed,
        });
    }

    /// The edit did not happen (the world refused it): forget the intent, change nothing else.
    pub fn abandon_pending(&mut self) {
        self.pending = None;
    }

    /// The world confirmed the edit: apply its effect and clear the intent in one step (they are
    /// saved together). Returns the objective message, if any.
    pub fn finish_pending(&mut self) -> Option<String> {
        match self.pending.take()?.kind {
            PendingKind::Mined { material } => self.on_mined(material),
            PendingKind::Placed => self.on_placed(),
        }
    }

    /// After a restart: settles an edit that was under way when the game stopped. `confirmed`
    /// says whether the world has the operation with that sequence. Confirmed: the effect is
    /// applied exactly once (the intent is cleared with it, so asking again changes nothing).
    /// Not confirmed: the world never changed, so only the intent is dropped. Save the state
    /// afterwards.
    pub fn reconcile(&mut self, confirmed: impl Fn(u64) -> bool) -> Reconciled {
        let Some(p) = self.pending else {
            return Reconciled::Nothing;
        };
        if confirmed(p.seq) {
            self.finish_pending();
            Reconciled::Applied
        } else {
            self.pending = None;
            Reconciled::Dropped
        }
    }

    /// Text form (one `key value...` line each, version first).
    pub fn encode(&self) -> String {
        let mut s = String::from("cube-game 1\n");
        s += &format!(
            "mode {}\n",
            if self.mode == Mode::Creative {
                "creative"
            } else {
                "survival"
            }
        );
        s += &format!("selected {}\n", self.inventory.selected());
        for (i, slot) in self.inventory.slots().iter().enumerate() {
            if let Some(st) = slot {
                s += &format!("slot {i} {} {}\n", st.item.0, st.count);
            }
        }
        let o = self.objective;
        s += &format!(
            "objective {} {} {} {} {}\n",
            Objective::ID,
            o.mine_goal,
            o.place_goal,
            o.mined,
            o.placed
        );
        s += &format!("health {} {}\n", self.health.cur(), self.health.max());
        if self.progress.xp() > 0 {
            s += &format!(
                "xp {}
",
                self.progress.xp()
            );
        }
        for (cx, cz) in &self.killed {
            s += &format!("kill {cx} {cz}\n");
        }
        for (x, z) in &self.looted {
            s += &format!("cache {x} {z}\n");
        }
        match self.pending {
            Some(PendingEdit {
                seq,
                kind: PendingKind::Mined { material },
            }) => s += &format!("pending {seq} mined {material}\n"),
            Some(PendingEdit {
                seq,
                kind: PendingKind::Placed,
            }) => s += &format!("pending {seq} placed\n"),
            None => {}
        }
        s
    }

    /// Reads [`GameState::encode`] output; `None` for anything that is not a complete, valid state.
    pub fn decode(text: &str) -> Option<Self> {
        let mut lines = text.lines();
        if lines.next()? != "cube-game 1" {
            return None;
        }
        let mut st = GameState::default();
        let (mut mode, mut objective, mut health_seen) = (false, false, false);
        let mut xp_seen = false;
        for l in lines {
            let mut it = l.split_whitespace();
            match it.next()? {
                "mode" => {
                    st.mode = match it.next()? {
                        "creative" => Mode::Creative,
                        "survival" => Mode::Survival,
                        _ => return None,
                    };
                    mode = true;
                }
                "selected" => {
                    let i: usize = it.next()?.parse().ok()?;
                    if i >= HOTBAR {
                        return None;
                    }
                    st.inventory.selected = i;
                }
                "slot" => {
                    let i: usize = it.next()?.parse().ok()?;
                    let item = ItemId(it.next()?.parse().ok()?);
                    let count: u32 = it.next()?.parse().ok()?;
                    let def = definition(item)?;
                    if i >= SLOTS
                        || count == 0
                        || count > def.max_stack
                        || st.inventory.slots[i].is_some()
                    {
                        return None;
                    }
                    st.inventory.slots[i] = Some(ItemStack { item, count });
                }
                "objective" => {
                    if it.next()? != Objective::ID {
                        return None;
                    }
                    let n: Vec<u32> = it.map(str::parse).collect::<Result<_, _>>().ok()?;
                    let [mine_goal, place_goal, mined, placed] = <[u32; 4]>::try_from(n).ok()?;
                    st.objective = Objective {
                        mine_goal,
                        place_goal,
                        mined,
                        placed,
                    };
                    objective = true;
                }
                "kill" => {
                    let cx: i64 = it.next()?.parse().ok()?;
                    let cz: i64 = it.next()?.parse().ok()?;
                    if it.next().is_some() || !st.killed.insert((cx, cz)) {
                        return None;
                    }
                }
                "cache" => {
                    let x: i64 = it.next()?.parse().ok()?;
                    let z: i64 = it.next()?.parse().ok()?;
                    if it.next().is_some() || !st.looted.insert((x, z)) {
                        return None;
                    }
                }
                "xp" => {
                    if xp_seen {
                        return None;
                    }
                    let xp: u32 = it.next()?.parse().ok()?;
                    if it.next().is_some() {
                        return None;
                    }
                    st.progress = crate::progression::Progress::from_xp(xp)?;
                    xp_seen = true;
                }
                "health" => {
                    if health_seen {
                        return None;
                    }
                    let cur: u32 = it.next()?.parse().ok()?;
                    let max: u32 = it.next()?.parse().ok()?;
                    if it.next().is_some() {
                        return None;
                    }
                    st.health = crate::combat::Health::from_parts(cur, max)?;
                    health_seen = true;
                }
                "pending" => {
                    if st.pending.is_some() {
                        return None;
                    }
                    let seq: u64 = it.next()?.parse().ok()?;
                    let kind = match it.next()? {
                        "mined" => {
                            let material: u16 = it.next()?.parse().ok()?;
                            definition(ItemId(material))?.places?;
                            PendingKind::Mined { material }
                        }
                        "placed" => PendingKind::Placed,
                        _ => return None,
                    };
                    if it.next().is_some() {
                        return None;
                    }
                    st.pending = Some(PendingEdit { seq, kind });
                }
                _ => return None,
            }
        }
        (mode && objective).then_some(st)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_fills_stacks_then_slots_and_reports_what_did_not_fit() {
        let mut inv = Inventory::new();
        assert_eq!(inv.add(ItemId(1), 70), 0);
        assert_eq!(
            inv.slots()[0],
            Some(ItemStack {
                item: ItemId(1),
                count: 64
            })
        );
        assert_eq!(
            inv.slots()[1],
            Some(ItemStack {
                item: ItemId(1),
                count: 6
            })
        );
        assert_eq!(inv.add(ItemId(1), 60), 0);
        assert_eq!(inv.count_of(ItemId(1)), 130);
        assert_eq!(inv.add(ItemId(99), 5), 5, "an unknown item never fits");
        let mut full = Inventory::new();
        assert_eq!(full.add(ItemId(2), SLOTS as u32 * MAX_STACK + 10), 10);
        assert!(!full.can_add(ItemId(2), 1));
        assert!(!full.can_add(ItemId(3), 1));
    }

    #[test]
    fn selection_wraps_and_stays_in_the_hotbar() {
        let mut inv = Inventory::new();
        inv.scroll(-1);
        assert_eq!(inv.selected(), HOTBAR - 1);
        inv.scroll(2);
        assert_eq!(inv.selected(), 1);
        inv.select(HOTBAR);
        assert_eq!(inv.selected(), 1);
        inv.select(4);
        assert_eq!(inv.selected(), 4);
    }

    #[test]
    fn taking_the_last_item_empties_the_slot_and_an_empty_slot_gives_nothing() {
        let mut inv = Inventory::new();
        inv.add(ItemId(3), 1);
        assert_eq!(inv.take_selected(), Some(ItemId(3)));
        assert_eq!(inv.slots()[0], None);
        assert_eq!(inv.take_selected(), None);
    }

    #[test]
    fn swap_moves_and_merges_stacks() {
        let mut inv = Inventory::new();
        inv.add(ItemId(1), 10);
        inv.add(ItemId(2), 5);
        inv.swap(0, 1);
        assert_eq!(inv.slots()[0].unwrap().item, ItemId(2));
        inv.swap(0, 20);
        assert_eq!(inv.slots()[20].unwrap().item, ItemId(2));
        assert!(inv.slots()[0].is_none());
        let mut inv = Inventory::new();
        inv.slots[0] = Some(ItemStack {
            item: ItemId(1),
            count: 60,
        });
        inv.slots[5] = Some(ItemStack {
            item: ItemId(1),
            count: 10,
        });
        inv.swap(5, 0);
        assert_eq!(inv.slots()[0].unwrap().count, 64);
        assert_eq!(inv.slots()[5].unwrap().count, 6);
        assert_eq!(
            inv.count_of(ItemId(1)),
            70,
            "merging never creates or loses items"
        );
    }

    #[test]
    fn survival_mining_adds_and_building_consumes_creative_does_neither() {
        let mut g = GameState::default();
        assert_eq!(g.selected_material(), None, "an empty hand places nothing");
        assert!(g.can_keep(1));
        g.on_mined(1);
        g.on_mined(1);
        assert_eq!(g.inventory.count_of(ItemId(1)), 2);
        assert_eq!(g.selected_material(), Some(1));
        g.on_placed();
        assert_eq!(g.inventory.count_of(ItemId(1)), 1);
        g.on_placed();
        assert_eq!(g.selected_material(), None);
        g.mode = Mode::Creative;
        g.inventory.select(4);
        assert_eq!(g.selected_material(), Some(5));
        g.on_placed();
        assert_eq!(
            g.inventory.count_of(ItemId(5)),
            0,
            "creative consumes nothing"
        );
        assert!(g.can_keep(1), "creative never refuses to keep");
    }

    #[test]
    fn mining_is_refused_when_the_block_would_not_fit() {
        let mut g = GameState::default();
        g.inventory.add(ItemId(1), SLOTS as u32 * MAX_STACK);
        assert!(!g.can_keep(1));
        assert!(!g.can_keep(2));
    }

    #[test]
    fn the_objective_completes_once_and_announces_it_once() {
        let mut g = GameState::default();
        g.inventory.add(ItemId(2), 40);
        let mut msgs = Vec::new();
        for _ in 0..8 {
            msgs.extend(g.on_mined(2));
        }
        assert!(msgs.is_empty(), "mining alone does not finish it");
        for _ in 0..12 {
            msgs.extend(g.on_placed());
        }
        assert_eq!(msgs.len(), 1);
        assert!(g.objective.done());
        assert!(
            g.on_mined(2).is_none() && g.on_placed().is_none(),
            "no second announcement"
        );
    }

    #[test]
    fn an_intent_that_the_world_confirmed_is_applied_exactly_once() {
        let mut g = GameState::default();
        g.begin_mined(7, 2);
        assert_eq!(g.inventory.count_of(ItemId(2)), 0, "nothing yet");
        let text = g.encode();
        let mut after = GameState::decode(&text).expect("the intent survives the file");
        assert_eq!(after.reconcile(|s| s == 7), Reconciled::Applied);
        assert_eq!(after.inventory.count_of(ItemId(2)), 1);
        assert_eq!(after.objective.mined, 1);
        assert!(after.pending.is_none());
        // Settling again (a second restart before anything else happened) changes nothing.
        let saved_again = GameState::decode(&after.encode()).unwrap();
        let mut twice = saved_again.clone();
        assert_eq!(twice.reconcile(|_| true), Reconciled::Nothing);
        assert_eq!(twice, saved_again);
    }

    #[test]
    fn an_intent_the_world_never_got_is_dropped_without_a_trace() {
        let mut g = GameState::default();
        g.inventory.add(ItemId(1), 3);
        g.begin_placed(9);
        let before = GameState {
            pending: None,
            ..g.clone()
        };
        assert_eq!(g.reconcile(|_| false), Reconciled::Dropped);
        assert_eq!(g, before, "no item was used, no objective progress");
    }

    #[test]
    fn a_placed_intent_uses_one_item_only_when_confirmed() {
        let mut g = GameState::default();
        g.inventory.add(ItemId(1), 2);
        g.begin_placed(4);
        assert_eq!(g.inventory.count_of(ItemId(1)), 2);
        assert_eq!(g.reconcile(|s| s == 4), Reconciled::Applied);
        assert_eq!(g.inventory.count_of(ItemId(1)), 1);
        assert_eq!(g.objective.placed, 1);
    }

    #[test]
    fn a_refused_edit_abandons_the_intent_and_costs_nothing() {
        let mut g = GameState::default();
        g.inventory.add(ItemId(1), 2);
        let before = g.clone();
        g.begin_placed(1);
        g.abandon_pending();
        assert_eq!(g, before);
    }

    #[test]
    fn health_is_saved_and_old_files_load_with_full_health() {
        let mut g = GameState::default();
        g.health.damage(7);
        let text = g.encode();
        let back = GameState::decode(&text).unwrap();
        assert_eq!(back.health.cur(), 13);
        assert_eq!(back, g);
        let old: String = text
            .lines()
            .filter(|l| !l.starts_with("health"))
            .map(|l| format!("{l}\n"))
            .collect();
        let loaded = GameState::decode(&old).expect("a file from before combat");
        assert_eq!(loaded.health.cur(), crate::combat::PLAYER_MAX_HEALTH);
        assert!(GameState::decode(&text.replace("health 13 20", "health 21 20")).is_none());
        assert!(GameState::decode(&text.replace("health 13 20", "health 13 0")).is_none());
        assert!(GameState::decode(&format!("{text}health 1 2\n")).is_none());
    }

    #[test]
    fn loot_goes_to_the_inventory_in_survival_and_never_in_creative() {
        let mut g = GameState::default();
        assert_eq!(g.grant_loot(ItemId(6), 2), 0);
        assert_eq!(g.inventory.count_of(ItemId(6)), 2);
        let mut c = GameState {
            mode: Mode::Creative,
            ..GameState::default()
        };
        assert_eq!(c.grant_loot(ItemId(6), 2), 0);
        assert_eq!(
            c.inventory.count_of(ItemId(6)),
            0,
            "creative creates nothing"
        );
    }

    #[test]
    fn loot_that_does_not_fit_is_reported_and_a_non_block_item_cannot_be_placed() {
        let mut g = GameState::default();
        g.inventory.add(ItemId(2), SLOTS as u32 * MAX_STACK);
        assert_eq!(g.grant_loot(ItemId(6), 3), 3, "full: all three left over");
        let mut h = GameState::default();
        h.inventory.add(ItemId(6), 1);
        assert_eq!(h.selected_material(), None, "gel is not a block");
        let text = h.encode();
        assert_eq!(
            GameState::decode(&text),
            Some(h),
            "the new item saves and loads"
        );
    }

    #[test]
    fn kills_are_saved_with_the_state_and_damage_is_detected() {
        let mut g = GameState::default();
        g.killed.insert((3, -2));
        g.killed.insert((-7, 9));
        g.grant_loot(ItemId(6), 2);
        let text = g.encode();
        assert_eq!(GameState::decode(&text), Some(g));
        assert!(
            GameState::decode(&format!(
                "{text}kill 3 -2
"
            ))
            .is_none(),
            "twice"
        );
        assert!(
            GameState::decode(&format!(
                "{text}kill 3
"
            ))
            .is_none()
        );
        assert!(
            GameState::decode(&format!(
                "{text}kill 3 4 5
"
            ))
            .is_none()
        );
    }

    #[test]
    fn a_save_written_before_reconcile_and_combat_still_loads() {
        // The exact format of the first game files (no pending, health or kill lines).
        let old = "cube-game 1
mode survival
selected 2
slot 0 1 64
slot 1 3 5
objective shelter-1 8 12 3 4
";
        let g = GameState::decode(old).expect("an old file loads");
        assert_eq!(g.inventory.count_of(ItemId(1)), 64);
        assert_eq!(g.inventory.count_of(ItemId(3)), 5);
        assert_eq!(g.inventory.selected(), 2);
        assert_eq!((g.objective.mined, g.objective.placed), (3, 4));
        assert_eq!(g.health.cur(), crate::combat::PLAYER_MAX_HEALTH);
        assert!(g.pending.is_none() && g.killed.is_empty());
        assert_eq!(
            GameState::decode(&g.encode()),
            Some(g),
            "and saves in the new format"
        );
    }

    #[test]
    fn a_damaged_intent_line_means_a_damaged_file() {
        let mut g = GameState::default();
        g.begin_mined(3, 1);
        let text = g.encode();
        assert!(GameState::decode(&text).is_some());
        assert!(GameState::decode(&text.replace("mined 1", "mined 77")).is_none());
        assert!(GameState::decode(&text.replace("pending 3", "pending x")).is_none());
        assert!(GameState::decode(&format!("{text}pending 4 placed\n")).is_none());
        assert!(GameState::decode(&text.replace("mined 1", "mined 1 9")).is_none());
    }

    #[test]
    fn save_text_round_trips_and_rejects_damage() {
        let mut g = GameState::default();
        g.inventory.add(ItemId(1), 70);
        g.inventory.add(ItemId(4), 3);
        g.inventory.select(2);
        g.mode = Mode::Creative;
        g.objective.mined = 5;
        g.objective.placed = 2;
        let text = g.encode();
        assert_eq!(GameState::decode(&text), Some(g));
        assert!(GameState::decode("garbage").is_none());
        assert!(
            GameState::decode(&text.replace("slot 0 1 64", "slot 0 1 99")).is_none(),
            "stack above the maximum"
        );
        assert!(
            GameState::decode(&text.replace("slot 1 1 6", "slot 0 1 6")).is_none(),
            "two stacks in one slot"
        );
        assert!(
            GameState::decode(&text.replace("mode creative\n", "")).is_none(),
            "a missing line"
        );
        assert!(GameState::decode(&text.replace("selected 2", "selected 40")).is_none());
        assert!(
            GameState::decode(&text.replace("slot 0 1 64", "slot 0 77 64")).is_none(),
            "unknown item"
        );
    }

    #[test]
    fn experience_is_saved_old_files_load_with_none_and_bad_values_are_refused() {
        let mut g = GameState::default();
        g.progress.gain(37);
        let text = g.encode();
        assert!(text.contains("xp 37"));
        let back = GameState::decode(&text).unwrap();
        assert_eq!(back.progress.xp(), 37);
        assert_eq!(back.progress.level(), 3);
        let old = text
            .lines()
            .filter(|l| !l.starts_with("xp"))
            .collect::<Vec<_>>();
        assert_eq!(
            GameState::decode(
                &(old.join(
                    "
"
                ) + "
")
            )
            .unwrap()
            .progress
            .xp(),
            0
        );
        assert!(
            GameState::decode(&format!(
                "{text}xp 5
"
            ))
            .is_none(),
            "twice"
        );
        assert!(GameState::decode(&text.replace("xp 37", "xp 99999")).is_none());
        assert!(GameState::decode(&text.replace("xp 37", "xp x")).is_none());
        assert!(GameState::decode(&text.replace("xp 37", "xp 3 4")).is_none());
    }

    #[test]
    fn taken_caches_are_saved_and_a_cache_cannot_be_listed_twice() {
        let mut g = GameState::default();
        g.looted.insert((-95, -29));
        g.looted.insert((12, 7));
        let text = g.encode();
        assert_eq!(GameState::decode(&text).unwrap().looted, g.looted);
        assert!(
            GameState::decode(&format!(
                "{text}cache 12 7
"
            ))
            .is_none()
        );
        assert!(
            GameState::decode(&format!(
                "{text}cache 1
"
            ))
            .is_none()
        );
        assert!(
            GameState::decode(&format!(
                "{text}cache 1 2 3
"
            ))
            .is_none()
        );
        let old: String = text
            .lines()
            .filter(|l| !l.starts_with("cache"))
            .map(|l| {
                format!(
                    "{l}
"
                )
            })
            .collect();
        assert!(GameState::decode(&old).unwrap().looted.is_empty());
    }

    #[test]
    fn crafting_consumes_inputs_makes_the_output_and_refuses_cleanly() {
        let mut g = GameState::default();
        assert_eq!(g.craft(0), Err(CraftRefused::Missing));
        assert_eq!(g.craft(5), Err(CraftRefused::Unknown));
        g.inventory.add(ItemId(12), 3);
        g.inventory.add(ItemId(8), 1);
        let before = g.inventory.clone();
        assert_eq!(g.craft(0), Err(CraftRefused::Missing));
        assert_eq!(g.inventory, before, "a refused craft changes nothing");
        g.inventory.add(ItemId(8), 1);
        assert_eq!(g.craft(0), Ok(()));
        assert_eq!(g.inventory.count_of(ItemId(12)), 0);
        assert_eq!(g.inventory.count_of(ItemId(8)), 0);
        assert_eq!(g.inventory.count_of(IRON_SWORD_ITEM), 1);
        // The result must fit: a full inventory with the inputs in the only slots still works
        // (the inputs free room), a full one without room refuses.
        let mut full = GameState::default();
        for i in 0..SLOTS {
            full.inventory.slots[i] = Some(ItemStack {
                item: ItemId(1),
                count: 1,
            });
        }
        full.inventory.slots[0] = Some(ItemStack {
            item: ItemId(12),
            count: 3,
        });
        full.inventory.slots[1] = Some(ItemStack {
            item: ItemId(8),
            count: 2,
        });
        assert_eq!(full.craft(0), Ok(()), "inputs freed two slots");
        let mut stuck = GameState::default();
        for i in 0..SLOTS {
            stuck.inventory.slots[i] = Some(ItemStack {
                item: ItemId(1),
                count: 1,
            });
        }
        stuck.inventory.slots[0] = Some(ItemStack {
            item: ItemId(12),
            count: 4,
        });
        stuck.inventory.slots[1] = Some(ItemStack {
            item: ItemId(8),
            count: 4,
        });
        let before = stuck.inventory.clone();
        assert_eq!(
            stuck.craft(0),
            Err(CraftRefused::NoRoom),
            "leftovers keep both slots"
        );
        assert_eq!(stuck.inventory, before, "a refused craft changes nothing");
        let mut c = GameState {
            mode: Mode::Creative,
            ..GameState::default()
        };
        assert_eq!(c.craft(0), Err(CraftRefused::Creative));
    }

    #[test]
    fn the_selected_item_picks_the_weapon_and_remove_spans_stacks() {
        assert_eq!(weapon_of(None).name, "Wooden sword");
        assert_eq!(weapon_of(Some(ItemId(1))).name, "Wooden sword");
        assert_eq!(weapon_of(Some(IRON_SWORD_ITEM)).name, "Iron sword");
        let mut inv = Inventory::new();
        inv.add(ItemId(2), MAX_STACK + 5);
        assert!(
            !inv.remove(ItemId(2), MAX_STACK + 6),
            "too many: nothing removed"
        );
        assert_eq!(inv.count_of(ItemId(2)), MAX_STACK + 5);
        assert!(inv.remove(ItemId(2), MAX_STACK + 1));
        assert_eq!(inv.count_of(ItemId(2)), 4);
        let sword = GameState::default();
        assert!(
            definition(IRON_SWORD_ITEM).is_some_and(|d| d.max_stack == 1 && d.places.is_none())
        );
        let _ = sword;
    }
}
