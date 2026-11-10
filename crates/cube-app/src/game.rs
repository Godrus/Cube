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
    pub places: u16,
}

/// All items of the prototype.
pub const ITEMS: [ItemDefinition; 5] = [
    ItemDefinition {
        id: ItemId(1),
        name: "Stone",
        max_stack: MAX_STACK,
        places: 1,
    },
    ItemDefinition {
        id: ItemId(2),
        name: "Dirt",
        max_stack: MAX_STACK,
        places: 2,
    },
    ItemDefinition {
        id: ItemId(3),
        name: "Grass",
        max_stack: MAX_STACK,
        places: 3,
    },
    ItemDefinition {
        id: ItemId(4),
        name: "Sand",
        max_stack: MAX_STACK,
        places: 4,
    },
    ItemDefinition {
        id: ItemId(5),
        name: "Red brick",
        max_stack: MAX_STACK,
        places: 5,
    },
];

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
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            inventory: Inventory::new(),
            mode: Mode::Survival,
            objective: Objective::SHELTER,
            pending: None,
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
                (i < ITEMS.len()).then(|| ITEMS[i].places)
            }
            Mode::Survival => self
                .inventory
                .selected_item()
                .and_then(definition)
                .map(|d| d.places),
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
        let (mut mode, mut objective) = (false, false);
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
                "pending" => {
                    if st.pending.is_some() {
                        return None;
                    }
                    let seq: u64 = it.next()?.parse().ok()?;
                    let kind = match it.next()? {
                        "mined" => {
                            let material: u16 = it.next()?.parse().ok()?;
                            definition(ItemId(material))?;
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
}
