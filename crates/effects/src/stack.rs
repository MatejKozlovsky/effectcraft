//! Per-thread state of the effect stack that is running: how the stack's buffer sits relative to
//! the buffer a blanket cover of the layer rectangle would give it, so CPU blurs round exactly as
//! they did on that buffer (#1205).

use std::cell::Cell;

thread_local! {
    /// Rows a blanket cover's buffer would have above the current buffer (0 with the cover).
    /// CPU blurs place their vertical block restarts with it, so they round exactly as on the
    /// taller buffer ([`StackScope`]).
    static ROW_PHASE: Cell<i64> = const { Cell::new(0) };
    /// The covered buffer in the current buffer's pixels (x, y, w, h), tracked through the
    /// stack's pads; `None` when the buffer was covered or not aligned. A Repeat Edge blur grows
    /// to it so it repeats the same edge pixels as the covered buffer ([`main_rect`]).
    static MAIN_RECT: Cell<Option<[i64; 4]>> = const { Cell::new(None) };
}

/// See [`MAIN_RECT`].
pub fn main_rect() -> Option<[i64; 4]> {
    MAIN_RECT.with(|c| c.get())
}

/// Set [`MAIN_RECT`].
pub fn set_main_rect(r: Option<[i64; 4]>) {
    MAIN_RECT.with(|c| c.set(r));
}

/// Both buffers grew by `pad` on every side ([`crate::Buf::pad`]).
pub(crate) fn main_rect_padded(pad: u32) {
    MAIN_RECT.with(|c| c.set(c.get().map(|[x, y, w, h]| [x, y, w + 2 * pad as i64, h + 2 * pad as i64])));
}

/// Only this buffer grew by `pad` on every side ([`crate::cover_pad`]; the covered one had the room).
pub fn main_rect_cover_pad(pad: u32) {
    let p = pad as i64;
    MAIN_RECT.with(|c| c.set(c.get().map(|[x, y, w, h]| [x + p, y + p, w - 2 * p, h - 2 * p])));
}

/// See [`ROW_PHASE`].
pub fn row_phase() -> i64 {
    ROW_PHASE.with(|c| c.get())
}

/// Set [`ROW_PHASE`] ([`crate::Buf::align`]).
pub fn set_row_phase(v: i64) {
    ROW_PHASE.with(|c| c.set(v));
}

/// The buffer grew `rows` on top that the covered buffer did not.
pub fn shift_row_phase(rows: i64) {
    ROW_PHASE.with(|c| c.set(c.get() - rows));
}

/// One layer's effect stack: the state starts empty and is restored afterwards (a nested layer's
/// stack can run inside an effect).
pub struct StackScope(i64, Option<[i64; 4]>);

impl StackScope {
    pub fn new() -> StackScope {
        StackScope(ROW_PHASE.with(|c| c.replace(0)), MAIN_RECT.with(|c| c.replace(None)))
    }
}

impl Default for StackScope {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for StackScope {
    fn drop(&mut self) {
        ROW_PHASE.with(|c| c.set(self.0));
        MAIN_RECT.with(|c| c.set(self.1));
    }
}
