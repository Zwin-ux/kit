//! Kit's fox — the 0.1 mascot, redrawn for the terminal.
//!
//! The 0.1 masters (`archive/ink-tui-final:assets/pixel/kit-frame-{1..6}.png`)
//! reduced to an 18×24 sprite and drawn two pixels per cell with half blocks,
//! so it is 18 columns by 12 rows. The body is locked; only the tail moves.
//! Every frame has the same canvas, so nothing around the fox shifts.
//!
//! Motion comes only from [`Clock`]: every [`WAG_PERIOD`] ticks the tail flicks
//! through five frames and comes back to rest. With motion off the fox rests.

use crate::event::Clock;

/// Columns the fox occupies.
pub const WIDTH: u16 = 18;
/// Rows the fox occupies.
pub const HEIGHT: u16 = 12;

/// Ticks between wags (8 s at 20 Hz).
pub const WAG_PERIOD: u64 = 160;
/// Ticks each wag frame holds (200 ms, the 0.1 pace).
const FRAME_TICKS: u64 = 4;

/// Resting pose, one string per pixel row (`#` is ink).
const REST: [&str; 24] = [
    "...#.........#....",
    "...##.......##....",
    "...###.....###....",
    "...####...####....",
    "...###########....",
    "...###########....",
    "...###########....",
    "..#############...",
    ".###############..",
    "#################.",
    ".###############..",
    "...###########....",
    "....#########.....",
    "....#############.",
    "...##############.",
    "..####.####.#####.",
    ".######.##.#######",
    ".#######..########",
    ".#################",
    ".#################",
    ".################.",
    "..###############.",
    "...#############..",
    ".....#########....",
];

/// Rows that differ from [`REST`] in each frame of the wag: the tail rises,
/// peaks, and settles. Frame 0 is the rest pose.
const TAIL: [&[(usize, &str)]; 6] = [
    &[],
    &[(12, "....#########..#.."), (13, "....##########.##.")],
    &[
        (11, "...###########..#."),
        (12, "....#########..##."),
        (13, "....##########.##."),
    ],
    &[
        (11, "...###########...#"),
        (12, "....#########..##."),
        (13, "....##########.##."),
    ],
    &[(12, "....#########..##."), (13, "....##########.##.")],
    &[(12, "....#########....."), (13, "....#########.###.")],
];

/// Number of frames in the wag, rest included.
pub const FRAMES: usize = TAIL.len();

/// The frame to draw at this tick. Always the rest pose when motion is off.
pub fn frame_at(clock: &Clock, motion: bool) -> usize {
    if !motion {
        return 0;
    }
    let wag = (FRAMES as u64 - 1) * FRAME_TICKS;
    let phase = clock.tick % WAG_PERIOD;
    let start = WAG_PERIOD - wag;
    if phase < start {
        0
    } else {
        1 + ((phase - start) / FRAME_TICKS) as usize
    }
}

/// True when the frame at `tick` differs from the frame at `tick - 1`, so an
/// idle screen redraws six times per wag and not on every tick.
pub fn changes_at(tick: u64) -> bool {
    let now = frame_at(&Clock { tick }, true);
    let before = frame_at(
        &Clock {
            tick: tick.wrapping_sub(1),
        },
        true,
    );
    now != before
}

/// The fox as [`HEIGHT`] strings of [`WIDTH`] cells each.
pub fn lines(frame: usize) -> Vec<String> {
    let mut rows = REST;
    for &(row, pixels) in TAIL[frame % FRAMES] {
        rows[row] = pixels;
    }
    rows.chunks(2)
        .map(|pair| {
            pair[0]
                .bytes()
                .zip(pair[1].bytes())
                .map(|(top, bottom)| match (top == b'#', bottom == b'#') {
                    (true, true) => '█',
                    (true, false) => '▀',
                    (false, true) => '▄',
                    (false, false) => ' ',
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_frame_keeps_the_same_canvas() {
        for f in 0..FRAMES {
            let l = lines(f);
            assert_eq!(l.len(), HEIGHT as usize);
            for row in &l {
                assert_eq!(row.chars().count(), WIDTH as usize, "frame {f}: {row:?}");
            }
        }
        for row in REST
            .iter()
            .chain(TAIL.iter().flat_map(|t| t.iter().map(|(_, r)| r)))
        {
            assert_eq!(row.len(), WIDTH as usize, "{row:?}");
        }
    }

    #[test]
    fn only_the_tail_moves() {
        let rest = lines(0);
        for f in 1..FRAMES {
            let l = lines(f);
            assert_ne!(l, rest, "frame {f} must differ from rest");
            // Ears, head and cheeks (the top five rows) never move.
            assert_eq!(l[..5], rest[..5], "frame {f} moved the head");
            // Chest and body (the bottom five rows) never move.
            assert_eq!(l[7..], rest[7..], "frame {f} moved the body");
        }
    }

    #[test]
    fn motion_off_always_rests() {
        for tick in 0..2 * WAG_PERIOD {
            assert_eq!(frame_at(&Clock { tick }, false), 0);
        }
    }

    #[test]
    fn one_wag_per_period_then_rest() {
        let frames: Vec<usize> = (0..WAG_PERIOD)
            .map(|tick| frame_at(&Clock { tick }, true))
            .collect();
        assert_eq!(frames[0], 0);
        assert_eq!(frames[WAG_PERIOD as usize - 1], FRAMES - 1);
        let mut seen: Vec<usize> = frames.clone();
        seen.dedup();
        assert_eq!(seen, vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(frame_at(&Clock { tick: WAG_PERIOD }, true), 0);
    }

    #[test]
    fn redraws_only_when_the_frame_changes() {
        let changes = (1..=WAG_PERIOD).filter(|&t| changes_at(t)).count();
        assert_eq!(changes, FRAMES, "one redraw per frame, rest included");
    }
}
