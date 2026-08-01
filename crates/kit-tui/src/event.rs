//! The single event stream — and the single clock.
//!
//! Kit 0.1's motion read as jitter because eight components each owned a
//! private timer and re-rendered independently. 1.0 has exactly one source of
//! time: `AppEvent::AnimationTick`. See `docs/dev/PRD-1.0.md` section 5.2.
//!
//! **No module outside this event loop may own a timer.** Motion is a pure
//! function of `tick`, so every animation in the app lands on the same beat.
//!
//! CONTRACT FILE. Changing anything here is a Claude-only operation; agents
//! file an issue instead of editing. See `docs/dev/BUILD-ASSIGNMENT.md`.

use crossterm::event::{KeyEvent, MouseEvent};
use kit_core::{GateOutcome, RunDelta, RunId};

/// Animation cadence. One tick per frame; motion derives from the counter.
pub const TICK_HZ: u64 = 20;
pub const TICK_INTERVAL: std::time::Duration = std::time::Duration::from_millis(1000 / TICK_HZ);

/// Everything that can move the application forward.
///
/// The event loop merges three sources into this one enum: the crossterm
/// event stream, one animation interval, and the run update channel.
#[derive(Debug, Clone)]
pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),

    /// The only source of motion in the entire application.
    AnimationTick,

    /// Incremental progress from a run task.
    RunUpdate(RunId, RunDelta),

    /// A gate finished. Separate from `RunUpdate` because it is the moment
    /// the Control Room's `GATE` column becomes meaningful.
    GateResult(RunId, GateOutcome),

    /// A background operation failed in a way the user must see.
    Error(String),

    /// Quit requested. The loop drains, restores the terminal, and exits.
    Quit,
}

impl AppEvent {
    /// Whether this event can change what is drawn.
    ///
    /// The loop redraws on state change, not on every tick — an idle Kit must
    /// sit under 1% CPU (M0 kill criterion).
    pub fn is_redraw_worthy(&self) -> bool {
        !matches!(self, Self::AnimationTick)
    }
}

/// Frame counter driving all motion.
///
/// Held by the app, advanced only by `AppEvent::AnimationTick`. Any animation
/// asks this for its phase rather than starting a timer of its own.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Clock {
    pub tick: u64,
}

impl Clock {
    pub fn advance(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    /// Index into an `n`-frame loop that advances every `every` ticks.
    pub fn frame(&self, n: usize, every: u64) -> usize {
        if n == 0 {
            return 0;
        }
        let every = every.max(1);
        ((self.tick / every) % n as u64) as usize
    }

    /// Square wave — cursors, pulses. `period` is in ticks.
    pub fn blink(&self, period: u64) -> bool {
        let period = period.max(1);
        (self.tick / period).is_multiple_of(2)
    }

    /// Whether a one-shot that started at `began` is still inside `duration` ticks.
    pub fn within(&self, began: u64, duration: u64) -> bool {
        self.tick.saturating_sub(began) < duration
    }
}

/// Honours `NO_COLOR` and `KIT_MOTION=off` (PRD definition of done).
///
/// When motion is disabled the loop still ticks, but animated components must
/// render their resting frame.
pub fn motion_enabled() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    !matches!(
        std::env::var("KIT_MOTION").as_deref(),
        Ok("off") | Ok("0") | Ok("false")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_cycles_and_never_divides_by_zero() {
        let c = Clock { tick: 7 };
        assert_eq!(c.frame(0, 1), 0);
        assert_eq!(c.frame(4, 0), 3);
        assert_eq!(Clock { tick: 8 }.frame(4, 2), 0);
    }

    #[test]
    fn blink_is_a_square_wave() {
        assert!(Clock { tick: 0 }.blink(5));
        assert!(Clock { tick: 4 }.blink(5));
        assert!(!Clock { tick: 5 }.blink(5));
        assert!(Clock { tick: 10 }.blink(5));
    }

    #[test]
    fn within_is_saturating_at_the_origin() {
        assert!(Clock { tick: 3 }.within(10, 5));
        assert!(Clock { tick: 12 }.within(10, 5));
        assert!(!Clock { tick: 15 }.within(10, 5));
    }

    #[test]
    fn ticks_alone_do_not_force_a_redraw() {
        assert!(!AppEvent::AnimationTick.is_redraw_worthy());
        assert!(AppEvent::Quit.is_redraw_worthy());
    }

    #[test]
    fn clock_wraps_without_panicking() {
        let mut c = Clock { tick: u64::MAX };
        c.advance();
        assert_eq!(c.tick, 0);
    }
}
