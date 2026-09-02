//! What a crossing IS: the type it names, the conversion for it, and which way
//! it goes.

use super::*;

/// One type-table cell: what the key names, and the adapter's answer for it.
pub(crate) struct TypeCell {
    /// The frontend's reading of this type, reused whole — so its classification
    /// and its origin are already here rather than re-derived per consumer.
    ///
    /// **Every** cell has one. There used to be a second variant for "a type only
    /// the binding authored", on the assumption that a declared wire type or a
    /// decomposition leaf had no reading to give. It did:
    /// those are ordinary types in this language, they were simply absent from an
    /// index of what the *source* wrote. `ensure_entry` takes the reading from the
    /// grammar when the cell is born and stores it right here, so it is always
    /// present, and a spelling the grammar genuinely refuses is a
    /// [`ScanError::NotExpressible`] naming it rather than a cell that quietly means
    /// less than its neighbours.
    pub subject: Box<prebindgen_flat::flat::TypeRef>,
    /// The binding asks for this cell **directly** — a declared fn's signature, a
    /// declared type, a decomposition leaf — as opposed to reaching it through
    /// another crossing's edges.
    ///
    /// A scan fact. Whether a converter is *needed* here is reachability from
    /// these roots, which [`crate::resolve`] derives rather than
    /// stores: the scan deliberately over-approximates the table (every nested
    /// position, every struct in both directions), so the roots are what say
    /// which of it has to work.
    pub root: bool,
    /// The crossings this one's conversion is built out of, once the adapter
    /// has answered.
    ///
    /// `Some` means the adapter has a conversion for it. The conversion itself
    /// stays in the adapter, which is the only thing that reads one; what the
    /// registry keeps is the edges, because what it does with an answer is walk
    /// it — the resolver follows them to decide which crossings a binding
    /// actually has to be able to make.
    ///
    /// **Identities, not spellings**, so `T`, `&T` and `Box<T>` reach one cell.
    pub entry: Option<Vec<TypeKey>>,
}

pub use crate::recipe::Direction;
