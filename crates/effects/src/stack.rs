//! Per-thread state of the effect stack that is running: how the stack's buffer sits relative to
//! the buffer a blanket cover of the layer rectangle would give it, so CPU blurs round exactly as
//! they did on that buffer (#1205).

use std::cell::Cell;

thread_local! {
    /// Rows a blanket cover's buffer would have above the current buffer (0 with the cover).
    /// CPU blurs place their vertical block restarts with it, so they round exactly as on the
    /// taller buffer ([`StackScope`]).
    static ROW_PHASE: Cell<i64> = const { Cell::new(0) };
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
pub struct StackScope(i64);

impl StackScope {
    pub fn new() -> StackScope {
        StackScope(ROW_PHASE.with(|c| c.replace(0)))
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
    }
}
