//! `lau-replay` — replay/timeline system for the Lau platform.
//!
//! Kids can rewind their adventure and watch it unfold again.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// TimelineEvent
// ---------------------------------------------------------------------------

/// Every event that can happen in a Lau adventure.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TimelineEvent {
    /// A block was placed in the world.
    BlockPlaced {
        x: i32,
        y: i32,
        z: i32,
        material: String,
        agent: String,
    },
    /// A block was removed from the world.
    BlockRemoved {
        x: i32,
        y: i32,
        z: i32,
        agent: String,
    },
    /// An agent moved from one position to another.
    AgentMoved {
        agent: String,
        from: (f64, f64, f64),
        to: (f64, f64, f64),
    },
    /// An agent said something.
    AgentSpoke {
        agent: String,
        text: String,
    },
    /// The ambient state of a room changed.
    RoomStateChanged {
        room: String,
        vibe: f64,
        phase: String,
    },
    /// A new quest was started.
    QuestStarted {
        quest: String,
        player: String,
    },
    /// A quest was completed.
    QuestCompleted {
        quest: String,
        player: String,
        time_ticks: u64,
    },
    /// A commit was made on a git-like branch.
    GitCommit {
        hash: String,
        message: String,
        author: String,
    },
    /// A new branch was created.
    BranchCreated {
        name: String,
        from_hash: String,
    },
    /// A branch was merged into the main line.
    Merged {
        branch: String,
        into_hash: String,
    },
}

impl TimelineEvent {
    /// A human-readable label for the type of event.
    /// Useful for grouping and analytics (e.g. `events_by_type`).
    pub fn type_label(&self) -> &'static str {
        match self {
            TimelineEvent::BlockPlaced { .. } => "BlockPlaced",
            TimelineEvent::BlockRemoved { .. } => "BlockRemoved",
            TimelineEvent::AgentMoved { .. } => "AgentMoved",
            TimelineEvent::AgentSpoke { .. } => "AgentSpoke",
            TimelineEvent::RoomStateChanged { .. } => "RoomStateChanged",
            TimelineEvent::QuestStarted { .. } => "QuestStarted",
            TimelineEvent::QuestCompleted { .. } => "QuestCompleted",
            TimelineEvent::GitCommit { .. } => "GitCommit",
            TimelineEvent::BranchCreated { .. } => "BranchCreated",
            TimelineEvent::Merged { .. } => "Merged",
        }
    }
}

// ---------------------------------------------------------------------------
// Timeline
// ---------------------------------------------------------------------------

/// An ordered sequence of timeline events with playback control.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Timeline {
    pub events: Vec<TimelineEvent>,
    pub cursor: usize,
}

impl Timeline {
    /// Create a new, empty timeline.
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            cursor: 0,
        }
    }

    /// Return the number of recorded events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns `true` when the timeline contains no events.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Record a new event at the end of the timeline.
    pub fn record(&mut self, event: TimelineEvent) {
        self.events.push(event);
    }

    /// Advance one step forward and return a reference to the event, or `None`
    /// if already at the end.
    pub fn play(&mut self) -> Option<&TimelineEvent> {
        if self.cursor >= self.events.len() {
            return None;
        }
        let ev = &self.events[self.cursor];
        self.cursor += 1;
        Some(ev)
    }

    /// Go back one step and return a reference to the *previous* event, or
    /// `None` if already at the start.
    pub fn rewind(&mut self) -> Option<&TimelineEvent> {
        if self.cursor == 0 {
            return None;
        }
        self.cursor -= 1;
        Some(&self.events[self.cursor])
    }

    /// Jump to the event whose index equals `tick` (0-based).
    ///
    /// If `tick` is beyond the last event, the cursor is set to the end so
    /// that subsequent calls to [`play`](Timeline::play) return `None`.
    ///
    /// Returns a reference to the event at the new cursor position when it
    /// exists (or the last event if clamped to end).
    pub fn seek(&mut self, tick: u64) -> Option<&TimelineEvent> {
        let idx = tick as usize;
        if idx >= self.events.len() {
            self.cursor = self.events.len();
            return self.events.last();
        }
        self.cursor = idx;
        Some(&self.events[idx])
    }

    /// Returns `true` when the cursor is at or past the last event.
    pub fn is_at_end(&self) -> bool {
        self.cursor >= self.events.len()
    }

    /// Returns `true` when the cursor is at the very start.
    pub fn is_at_start(&self) -> bool {
        self.cursor == 0
    }

    /// Total number of recorded events (ticks).
    pub fn total_ticks(&self) -> u64 {
        self.events.len() as u64
    }

    /// Build a [`TimelineSummary`] from the recorded events.
    pub fn summarize(&self) -> TimelineSummary {
        let mut events_by_type: HashMap<String, usize> = HashMap::new();
        let mut agents_active: Vec<String> = Vec::new();
        let mut rooms_visited: Vec<String> = Vec::new();
        let mut quests_completed: usize = 0;
        let mut blocks_placed: usize = 0;

        for event in &self.events {
            *events_by_type
                .entry(event.type_label().to_string())
                .or_insert(0) += 1;

            match event {
                TimelineEvent::BlockPlaced { agent, .. } => {
                    blocks_placed += 1;
                    if !agents_active.contains(agent) {
                        agents_active.push(agent.clone());
                    }
                }
                TimelineEvent::BlockRemoved { agent, .. } => {
                    if !agents_active.contains(agent) {
                        agents_active.push(agent.clone());
                    }
                }
                TimelineEvent::AgentMoved { agent, .. } => {
                    if !agents_active.contains(agent) {
                        agents_active.push(agent.clone());
                    }
                }
                TimelineEvent::AgentSpoke { agent, .. } => {
                    if !agents_active.contains(agent) {
                        agents_active.push(agent.clone());
                    }
                }
                TimelineEvent::RoomStateChanged { room, .. } => {
                    if !rooms_visited.contains(room) {
                        rooms_visited.push(room.clone());
                    }
                }
                TimelineEvent::QuestCompleted { .. } => {
                    quests_completed += 1;
                }
                TimelineEvent::QuestStarted { player, .. } => {
                    if !agents_active.contains(player) {
                        agents_active.push(player.clone());
                    }
                }
                TimelineEvent::GitCommit { .. }
                | TimelineEvent::BranchCreated { .. }
                | TimelineEvent::Merged { .. } => {}
            }
        }

        // Capture agents from AgentMoved/AgentSpoke already handled above.
        // Also collect rooms from *all* events that mention them.
        TimelineSummary {
            total_events: self.total_ticks() as usize,
            duration_ticks: self.total_ticks(),
            events_by_type,
            agents_active,
            rooms_visited,
            quests_completed,
            blocks_placed,
        }
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// TimelineSummary
// ---------------------------------------------------------------------------

/// Analytical summary of a timeline's contents.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimelineSummary {
    pub total_events: usize,
    pub duration_ticks: u64,
    pub events_by_type: HashMap<String, usize>,
    pub agents_active: Vec<String>,
    pub rooms_visited: Vec<String>,
    pub quests_completed: usize,
    pub blocks_placed: usize,
}

// ---------------------------------------------------------------------------
// Replay
// ---------------------------------------------------------------------------

/// Plays back a timeline at a configurable speed.
///
/// `speed` controls how many events are consumed per `play()` call:
/// - `1.0` → one event per call (normal speed)
/// - `2.0` → two events per call (double speed)
/// - `0.5` → one event every other call (half speed, uses an internal
///   fractional accumulator)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Replay {
    pub speed: f64,
    /// Fractional tick accumulator for sub-1.0 speeds.
    #[cfg_attr(feature = "serde", serde(skip))]
    accumulator: f64,
    timeline: Timeline,
}

impl Replay {
    /// Wrap a [`Timeline`] for playback at normal speed.
    pub fn new(timeline: Timeline) -> Self {
        Self {
            speed: 1.0,
            accumulator: 0.0,
            timeline,
        }
    }

    /// Wrap a [`Timeline`] for playback at the given speed.
    pub fn with_speed(timeline: Timeline, speed: f64) -> Self {
        Self {
            speed,
            accumulator: 0.0,
            timeline,
        }
    }

    /// Advance playback by `speed`-adjusted steps.
    ///
    /// At `speed = 1.0` this advances one event. At `speed = 2.0` it
    /// advances two. At `speed = 0.5` it only advances every other call.
    ///
    /// Returns the events consumed in this step (may be empty when at end
    /// or when the accumulator hasn't reached a full tick).
    pub fn play(&mut self) -> Vec<TimelineEvent> {
        let mut events = Vec::new();

        if self.timeline.is_at_end() {
            return events;
        }

        self.accumulator += self.speed;

        // Consume whole ticks from the accumulator.
        let count = self.accumulator.floor() as usize;
        self.accumulator -= count as f64;

        for _ in 0..count {
            if self.timeline.is_at_end() {
                break;
            }
            // We need to clone because play() returns a reference.
            if let Some(ev) = self.timeline.play() {
                events.push(ev.clone());
            }
        }

        events
    }

    /// Rewind one step (ignores speed — always goes back exactly one event).
    pub fn rewind(&mut self) -> Option<TimelineEvent> {
        self.timeline.rewind().cloned()
    }

    /// Jump to a specific tick.
    pub fn seek(&mut self, tick: u64) -> Option<TimelineEvent> {
        self.accumulator = 0.0;
        self.timeline.seek(tick).cloned()
    }

    /// Returns `true` if playback has reached the end of the timeline.
    pub fn is_at_end(&self) -> bool {
        self.timeline.is_at_end()
    }

    /// Returns `true` if playback is at the start of the timeline.
    pub fn is_at_start(&self) -> bool {
        self.timeline.is_at_start()
    }

    /// Immutable reference to the underlying timeline.
    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    /// Mutable reference to the underlying timeline.
    pub fn timeline_mut(&mut self) -> &mut Timeline {
        &mut self.timeline
    }

    /// Generate a kid-friendly narration string for a given event.
    pub fn narrate(&self, event: &TimelineEvent) -> String {
        match event {
            TimelineEvent::BlockPlaced {
                x,
                y,
                z,
                material,
                agent,
            } => {
                format!(
                    "{} placed a {} block at ({}, {}, {})! 🧱",
                    agent, material, x, y, z
                )
            }
            TimelineEvent::BlockRemoved { x, y, z, agent } => {
                format!("{} broke a block at ({}, {}, {}). 💥", agent, x, y, z)
            }
            TimelineEvent::AgentMoved { agent, from, to } => {
                format!(
                    "{} moved from ({:.1}, {:.1}, {:.1}) to ({:.1}, {:.1}, {:.1}). 🚶",
                    agent, from.0, from.1, from.2, to.0, to.1, to.2
                )
            }
            TimelineEvent::AgentSpoke { agent, text } => {
                format!("{} says: \"{}\" 💬", agent, text)
            }
            TimelineEvent::RoomStateChanged { room, vibe, phase } => {
                format!(
                    "The {} room feels different now — vibe {:.1}, phase '{}'. ✨",
                    room, vibe, phase
                )
            }
            TimelineEvent::QuestStarted { quest, player } => {
                format!("🌟 New quest for {}: \"{}\"!", player, quest)
            }
            TimelineEvent::QuestCompleted {
                quest,
                player,
                time_ticks,
            } => {
                format!(
                    "🎉 {} completed \"{}\" in {} ticks! Amazing!",
                    player, quest, time_ticks
                )
            }
            TimelineEvent::GitCommit {
                hash,
                message,
                author,
            } => {
                format!(
                    "{} committed '{}' (hash: {}) 📝",
                    author, message, &hash[..hash.len().min(7)]
                )
            }
            TimelineEvent::BranchCreated { name, from_hash } => {
                format!(
                    "New branch '{}' from {} 🌿",
                    name,
                    &from_hash[..from_hash.len().min(7)]
                )
            }
            TimelineEvent::Merged { branch, into_hash } => {
                format!(
                    "Branch '{}' merged into {} 🔀",
                    branch,
                    &into_hash[..into_hash.len().min(7)]
                )
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- helpers ----------

    fn sample_events() -> Vec<TimelineEvent> {
        vec![
            TimelineEvent::AgentMoved {
                agent: "Alex".into(),
                from: (0.0, 0.0, 0.0),
                to: (10.0, 0.0, 5.0),
            },
            TimelineEvent::RoomStateChanged {
                room: "Library".into(),
                vibe: 0.8,
                phase: "calm".into(),
            },
            TimelineEvent::BlockPlaced {
                x: 5,
                y: 3,
                z: 2,
                material: "stone".into(),
                agent: "Alex".into(),
            },
            TimelineEvent::AgentSpoke {
                agent: "Alex".into(),
                text: "Look at this cool castle!".into(),
            },
            TimelineEvent::QuestStarted {
                quest: "Find the Crystal".into(),
                player: "Alex".into(),
            },
            TimelineEvent::BlockPlaced {
                x: 6,
                y: 3,
                z: 2,
                material: "diamond".into(),
                agent: "Alex".into(),
            },
            TimelineEvent::QuestCompleted {
                quest: "Find the Crystal".into(),
                player: "Alex".into(),
                time_ticks: 42,
            },
            TimelineEvent::GitCommit {
                hash: "a1b2c3d4e5f6".into(),
                message: "Add crystal chamber".into(),
                author: "Lau".into(),
            },
            TimelineEvent::BranchCreated {
                name: "feature/crystal".into(),
                from_hash: "a1b2c3d4e5f6".into(),
            },
            TimelineEvent::Merged {
                branch: "feature/crystal".into(),
                into_hash: "f6e5d4c3b2a1".into(),
            },
        ]
    }

    // ---------- Timeline::new / is_empty ----------

    #[test]
    fn test_new_timeline_is_empty() {
        let tl = Timeline::new();
        assert!(tl.is_empty());
        assert!(tl.is_at_start());
        assert!(tl.is_at_end());
        assert_eq!(tl.total_ticks(), 0);
        assert_eq!(tl.len(), 0);
    }

    // ---------- Timeline::record ----------

    #[test]
    fn test_record_adds_event() {
        let mut tl = Timeline::new();
        tl.record(TimelineEvent::AgentSpoke {
            agent: "Bot".into(),
            text: "hello".into(),
        });
        assert_eq!(tl.len(), 1);
        assert_eq!(tl.total_ticks(), 1);
        assert!(!tl.is_empty());
    }

    // ---------- Timeline::play ----------

    #[test]
    fn test_play_advances_cursor() {
        let evs = sample_events();
        let mut tl = Timeline {
            events: evs.clone(),
            cursor: 0,
        };

        for (i, expected) in evs.iter().enumerate() {
            let got = tl.play().expect("should have event");
            assert_eq!(got, expected, "mismatch at index {i}");
        }
        assert!(tl.is_at_end());
        assert!(tl.play().is_none());
    }

    #[test]
    fn test_play_empty_timeline_returns_none() {
        let mut tl = Timeline::new();
        assert!(tl.play().is_none());
    }

    // ---------- Timeline::rewind ----------

    #[test]
    fn test_rewind_goes_back() {
        let mut tl = Timeline {
            events: sample_events(),
            cursor: 3,
        };

        let ev = tl.rewind().expect("should have previous event");
        assert_eq!(ev.type_label(), "BlockPlaced");
        assert_eq!(tl.cursor, 2);

        let ev = tl.rewind().unwrap();
        assert_eq!(ev.type_label(), "RoomStateChanged");
        assert_eq!(tl.cursor, 1);
    }

    #[test]
    fn test_rewind_at_start_returns_none() {
        let mut tl = Timeline {
            events: sample_events(),
            cursor: 0,
        };
        assert!(tl.rewind().is_none());
    }

    #[test]
    fn test_rewind_empty_timeline() {
        let mut tl = Timeline::new();
        assert!(tl.rewind().is_none());
    }

    // ---------- Timeline::seek ----------

    #[test]
    fn test_seek_to_specific_tick() {
        let mut tl = Timeline {
            events: sample_events(),
            cursor: 0,
        };

        let ev = tl.seek(2).expect("should have event at tick 2");
        assert_eq!(ev.type_label(), "BlockPlaced");
        assert_eq!(tl.cursor, 2);
    }

    #[test]
    fn test_seek_beyond_end_clamps() {
        let mut tl = Timeline {
            events: sample_events(),
            cursor: 0,
        };
        let n = tl.total_ticks();

        // seek past the end — should return last event and put cursor at end
        let ev = tl.seek(n + 10).expect("should clamp to last event");
        assert_eq!(ev.type_label(), "Merged");
        assert!(tl.is_at_end());
    }

    #[test]
    fn test_seek_to_zero() {
        let mut tl = Timeline {
            events: sample_events(),
            cursor: 5,
        };
        let ev = tl.seek(0).unwrap();
        assert_eq!(ev.type_label(), "AgentMoved");
        assert!(tl.is_at_start());
    }

    // ---------- Timeline::is_at_end / is_at_start ----------

    #[test]
    fn test_is_at_end_after_full_playback() {
        let mut tl = Timeline {
            events: sample_events(),
            cursor: 0,
        };
        while tl.play().is_some() {}
        assert!(tl.is_at_end());
        assert!(!tl.is_at_start());
    }

    #[test]
    fn test_is_at_start_initially() {
        let tl = Timeline::new();
        assert!(tl.is_at_start());
        assert!(tl.is_at_end());
    }

    // ---------- Timeline::summarize ----------

    #[test]
    fn test_summarize_counts() {
        let mut tl = Timeline {
            events: sample_events(),
            cursor: 0,
        };
        // Add a couple more events for variety
        tl.record(TimelineEvent::BlockRemoved {
            x: 5,
            y: 3,
            z: 2,
            agent: "Alex".into(),
        });
        tl.record(TimelineEvent::AgentSpoke {
            agent: "Bot".into(),
            text: "Nice work!".into(),
        });

        let summary = tl.summarize();
        assert_eq!(summary.total_events, 12);
        assert_eq!(summary.duration_ticks, 12);
        assert_eq!(summary.blocks_placed, 2);
        assert_eq!(summary.quests_completed, 1);
        assert!(summary.agents_active.contains(&"Alex".to_string()));
        assert!(summary.agents_active.contains(&"Bot".to_string()));
        assert!(summary.rooms_visited.contains(&"Library".to_string()));
        assert_eq!(*summary.events_by_type.get("BlockPlaced").unwrap(), 2);
        assert_eq!(*summary.events_by_type.get("BlockRemoved").unwrap(), 1);
        assert_eq!(*summary.events_by_type.get("AgentSpoke").unwrap(), 2);
    }

    #[test]
    fn test_summarize_empty() {
        let tl = Timeline::new();
        let summary = tl.summarize();
        assert_eq!(summary.total_events, 0);
        assert_eq!(summary.duration_ticks, 0);
        assert!(summary.agents_active.is_empty());
        assert!(summary.rooms_visited.is_empty());
        assert_eq!(summary.quests_completed, 0);
        assert_eq!(summary.blocks_placed, 0);
    }

    // ---------- Replay ----------

    #[test]
    fn test_replay_new_default_speed() {
        let tl = Timeline {
            events: sample_events(),
            cursor: 0,
        };
        let replay = Replay::new(tl);
        assert!((replay.speed - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_replay_with_speed() {
        let tl = Timeline {
            events: sample_events(),
            cursor: 0,
        };
        let replay = Replay::with_speed(tl, 2.5);
        assert!((replay.speed - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_replay_play_normal_speed() {
        let tl = Timeline {
            events: sample_events(),
            cursor: 0,
        };
        let mut replay = Replay::new(tl);
        let events = replay.play();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].type_label(), "AgentMoved");
        assert!(!replay.is_at_end());
    }

    #[test]
    fn test_replay_play_double_speed() {
        let tl = Timeline {
            events: sample_events(),
            cursor: 0,
        };
        let mut replay = Replay::with_speed(tl, 2.0);
        let events = replay.play();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].type_label(), "AgentMoved");
        assert_eq!(events[1].type_label(), "RoomStateChanged");
    }

    #[test]
    fn test_replay_play_half_speed() {
        let tl = Timeline {
            events: sample_events(),
            cursor: 0,
        };
        let mut replay = Replay::with_speed(tl, 0.5);

        // First call: accumulator = 0.5, floor = 0 → no events
        let events = replay.play();
        assert!(events.is_empty());

        // Second call: accumulator = 1.0, floor = 1 → one event
        let events = replay.play();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].type_label(), "AgentMoved");
    }

    #[test]
    fn test_replay_play_until_end() {
        let tl = Timeline {
            events: sample_events(),
            cursor: 0,
        };
        let mut replay = Replay::with_speed(tl, 3.0);
        let mut total = 0;
        loop {
            let batch = replay.play();
            if batch.is_empty() {
                break;
            }
            total += batch.len();
        }
        assert_eq!(total, 10);
        assert!(replay.is_at_end());
    }

    #[test]
    fn test_replay_rewind() {
        let tl = Timeline {
            events: sample_events(),
            cursor: 0,
        };
        let mut replay = Replay::new(tl);
        replay.play(); // cursor → 1
        replay.play(); // cursor → 2
        let ev = replay.rewind().unwrap();
        assert_eq!(ev.type_label(), "RoomStateChanged");
        let ev = replay.rewind().unwrap();
        assert_eq!(ev.type_label(), "AgentMoved");
        assert!(replay.rewind().is_none());
        assert!(replay.is_at_start());
    }

    #[test]
    fn test_replay_seek() {
        let tl = Timeline {
            events: sample_events(),
            cursor: 0,
        };
        let mut replay = Replay::with_speed(tl, 1.0);
        let ev = replay.seek(5).unwrap();
        assert_eq!(ev.type_label(), "BlockPlaced");
        // After seek, accumulator should be 0; next play gives event at tick 6
        let batch = replay.play();
        assert_eq!(batch.len(), 1);
        assert_eq!(batch[0].type_label(), "BlockPlaced");
        // play again for tick 6
        let batch = replay.play();
        assert_eq!(batch.len(), 1);
        assert_eq!(batch[0].type_label(), "QuestCompleted");
    }

    // ---------- Replay::narrate ----------

    #[test]
    fn test_narrate_block_placed() {
        let tl = Timeline::new();
        let replay = Replay::new(tl);
        let ev = TimelineEvent::BlockPlaced {
            x: 1,
            y: 2,
            z: 3,
            material: "wool".into(),
            agent: "Mia".into(),
        };
        let s = replay.narrate(&ev);
        assert!(s.contains("Mia"));
        assert!(s.contains("wool"));
        assert!(s.contains("1, 2, 3"));
        assert!(s.contains("🧱"));
    }

    #[test]
    fn test_narrate_block_removed() {
        let tl = Timeline::new();
        let replay = Replay::new(tl);
        let ev = TimelineEvent::BlockRemoved {
            x: 4,
            y: 5,
            z: 6,
            agent: "Mia".into(),
        };
        let s = replay.narrate(&ev);
        assert!(s.contains("broke"));
        assert!(s.contains("💥"));
    }

    #[test]
    fn test_narrate_agent_moved() {
        let tl = Timeline::new();
        let replay = Replay::new(tl);
        let ev = TimelineEvent::AgentMoved {
            agent: "Zoe".into(),
            from: (0.0, 0.0, 0.0),
            to: (3.5, 1.0, 2.0),
        };
        let s = replay.narrate(&ev);
        assert!(s.contains("Zoe"));
        assert!(s.contains("moved"));
        assert!(s.contains("3.5"));
    }

    #[test]
    fn test_narrate_agent_spoke() {
        let tl = Timeline::new();
        let replay = Replay::new(tl);
        let ev = TimelineEvent::AgentSpoke {
            agent: "Ella".into(),
            text: "Hello world!".into(),
        };
        let s = replay.narrate(&ev);
        assert!(s.contains("Ella"));
        assert!(s.contains("Hello world!"));
    }

    #[test]
    fn test_narrate_room_state_changed() {
        let tl = Timeline::new();
        let replay = Replay::new(tl);
        let ev = TimelineEvent::RoomStateChanged {
            room: "Kitchen".into(),
            vibe: 0.99,
            phase: "chaos".into(),
        };
        let s = replay.narrate(&ev);
        assert!(s.contains("Kitchen"));
        // 0.99 formatted with {:.1} = "1.0"
        assert!(s.contains("1.0") || s.contains("0.99"));
        assert!(s.contains("chaos"));
    }

    #[test]
    fn test_narrate_quest_started() {
        let tl = Timeline::new();
        let replay = Replay::new(tl);
        let ev = TimelineEvent::QuestStarted {
            quest: "Save the Village".into(),
            player: "Nico".into(),
        };
        let s = replay.narrate(&ev);
        assert!(s.contains("Nico"));
        assert!(s.contains("Save the Village"));
        assert!(s.contains("🌟"));
    }

    #[test]
    fn test_narrate_quest_completed() {
        let tl = Timeline::new();
        let replay = Replay::new(tl);
        let ev = TimelineEvent::QuestCompleted {
            quest: "Find the Key".into(),
            player: "Nico".into(),
            time_ticks: 1337,
        };
        let s = replay.narrate(&ev);
        assert!(s.contains("Nico"));
        assert!(s.contains("Find the Key"));
        assert!(s.contains("1337"));
        assert!(s.contains("🎉"));
    }

    #[test]
    fn test_narrate_git_commit() {
        let tl = Timeline::new();
        let replay = Replay::new(tl);
        let ev = TimelineEvent::GitCommit {
            hash: "deadbeef1234567".into(),
            message: "Fix the thing".into(),
            author: "Lau".into(),
        };
        let s = replay.narrate(&ev);
        assert!(s.contains("Lau"));
        assert!(s.contains("Fix the thing"));
        assert!(s.contains("deadbee"));
    }

    #[test]
    fn test_narrate_branch_created() {
        let tl = Timeline::new();
        let replay = Replay::new(tl);
        let ev = TimelineEvent::BranchCreated {
            name: "feature/magic".into(),
            from_hash: "aaaaaabbbbbb".into(),
        };
        let s = replay.narrate(&ev);
        assert!(s.contains("feature/magic"));
        // "aaaaaab" is the first 7 chars of "aaaaaabbbbbb" via .min(7)
        assert!(s.contains("aaaaaab"));
    }

    #[test]
    fn test_narrate_merged() {
        let tl = Timeline::new();
        let replay = Replay::new(tl);
        let ev = TimelineEvent::Merged {
            branch: "fix/crash".into(),
            into_hash: "ccccccdddddd".into(),
        };
        let s = replay.narrate(&ev);
        assert!(s.contains("fix/crash"));
        assert!(s.contains("ccccccd"));
    }

    // ---------- Event type_label ----------

    #[test]
    fn test_type_labels() {
        let cases: Vec<(TimelineEvent, &str)> = vec![
            (
                TimelineEvent::BlockPlaced {
                    x: 0,
                    y: 0,
                    z: 0,
                    material: "dirt".into(),
                    agent: "a".into(),
                },
                "BlockPlaced",
            ),
            (
                TimelineEvent::BlockRemoved {
                    x: 0,
                    y: 0,
                    z: 0,
                    agent: "a".into(),
                },
                "BlockRemoved",
            ),
            (
                TimelineEvent::AgentMoved {
                    agent: "a".into(),
                    from: (0., 0., 0.),
                    to: (1., 1., 1.),
                },
                "AgentMoved",
            ),
            (
                TimelineEvent::AgentSpoke {
                    agent: "a".into(),
                    text: "hi".into(),
                },
                "AgentSpoke",
            ),
            (
                TimelineEvent::RoomStateChanged {
                    room: "r".into(),
                    vibe: 1.0,
                    phase: "p".into(),
                },
                "RoomStateChanged",
            ),
            (
                TimelineEvent::QuestStarted {
                    quest: "q".into(),
                    player: "p".into(),
                },
                "QuestStarted",
            ),
            (
                TimelineEvent::QuestCompleted {
                    quest: "q".into(),
                    player: "p".into(),
                    time_ticks: 0,
                },
                "QuestCompleted",
            ),
            (
                TimelineEvent::GitCommit {
                    hash: "h".into(),
                    message: "m".into(),
                    author: "a".into(),
                },
                "GitCommit",
            ),
            (
                TimelineEvent::BranchCreated {
                    name: "n".into(),
                    from_hash: "h".into(),
                },
                "BranchCreated",
            ),
            (
                TimelineEvent::Merged {
                    branch: "b".into(),
                    into_hash: "h".into(),
                },
                "Merged",
            ),
        ];
        for (event, expected) in &cases {
            assert_eq!(event.type_label(), *expected);
        }
    }

    // ---------- Default ----------

    #[test]
    fn test_timeline_default() {
        let tl: Timeline = Default::default();
        assert!(tl.is_empty());
    }

    // ---------- Replay accessors ----------

    #[test]
    fn test_replay_timeline_ref() {
        let tl = Timeline {
            events: sample_events(),
            cursor: 0,
        };
        let replay = Replay::new(tl);
        assert_eq!(replay.timeline().len(), 10);
    }

    #[test]
    fn test_replay_timeline_mut() {
        let tl = Timeline {
            events: vec![],
            cursor: 0,
        };
        let mut replay = Replay::new(tl);
        replay
            .timeline_mut()
            .record(TimelineEvent::AgentSpoke {
                agent: "X".into(),
                text: "y".into(),
            });
        assert_eq!(replay.timeline().len(), 1);
    }
}
