//! The declared-class table, and the one door into it.
//!
//! # Why this is a module
//!
//! The table used to be a `pub(crate) HashMap<TypeKey, TypeConfig>` field on
//! [`Declarations`](super::Declarations), and every reader built its own key.
//! Building it from a **spelling** is wrong for a declaration and right for a
//! conversion, the difference is invisible at the call site, and the resulting
//! defects do not announce themselves: a `Box<Payload>` parameter found no
//! `Payload` declaration and silently lost its data-class lowering (#294), a
//! `Box<Priority>` field rendered as its `Int` wire instead of the Kotlin enum
//! class (#308), and `is_kotlin_enum` answered about the wrapper (#290).
//!
//! Making the field private would have enforced nothing: `Declarations` is
//! declared in `jni`, every consumer is a **descendant** module of `jni`, and
//! Rust makes a private item visible to descendants. Measured — the whole crate
//! still compiled. The boundary has to be a module the consumers are *outside*
//! of, which is what this file is. `map` is private to `decl_table`, so
//! `jni::trait_impl` and its siblings can reach the table only through the
//! accessors below.
//!
//! # The rule the accessors encode
//!
//! A **declaration** says what a type IS to Kotlin, and a wrapper the model
//! erases cannot change that: `Box<Payload>` is a `Payload` over there. So the
//! lookup strips, and it takes a [`TypeRef`] — the reading only the model can
//! mint (#280) — rather than a key the caller assembled. There is no key to
//! build, and therefore none to build wrongly.
//!
//! A **conversion** is the other question and keeps its wrappers, because
//! `Box<Option<T>>` and `Option<T>` genuinely need different converter bodies.
//! That lives in the registry's type tables, keyed by `TypeKey`, and is
//! deliberately untouched by this module.

use std::collections::HashMap;

use super::TypeConfig;
use crate::api::core::{flat::TypeRef, registry::TypeKey};

/// Every type declared to this adapter by a class declarator, keyed by the
/// **stripped** spelling.
///
/// Presence *is* "declared as a class" — the single writer is
/// `JniGenBuilder::register_class`, through [`Self::entry`].
#[derive(Clone, Default)]
pub(crate) struct DeclTable {
    /// Private to this module on purpose — see the module docs. Keyed by
    /// `TypeRef::stripped_key`, so a wrapped and a bare spelling of one type
    /// land on one entry.
    map: HashMap<TypeKey, TypeConfig>,
}

impl DeclTable {
    /// The declaration this type carries, **wrappers ignored**.
    ///
    /// The door for every property question — what kind is it, what is it
    /// called in Kotlin, is it a sum. Takes the reading, so the identity is
    /// derived here rather than asserted by the caller.
    ///
    /// A caller that needs a wrapped spelling to *decline* — converter
    /// selection, where finding `Priority` for `Box<Priority>` would emit a body
    /// that takes a bare `Priority` and not compile — says so in the open, with
    /// the same `erased_wrappers()` predicate the transparent bridges use:
    ///
    /// ```ignore
    /// if !ty.erased_wrappers().is_empty() {
    ///     return None; // the bridge serves this, and puts the wrapper back
    /// }
    /// let cfg = ext.declarations().declaration(ty)?;
    /// ```
    ///
    /// That is a routing decision and reads as one, rather than a second
    /// accessor whose name has to be remembered — which is how `key` and
    /// `stripped_key` came to disagree in the first place.
    pub(crate) fn declaration(&self, ty: &TypeRef) -> Option<&TypeConfig> {
        self.map.get(&ty.stripped_key())
    }

    /// [`Self::declaration`] for a caller that holds only a **spelling**.
    ///
    /// The visible "I only had tokens" step, exactly as
    /// [`Conversions::reading_of`](crate::api::core::registry::Conversions::reading_of)
    /// is for readings. It gives the SAME answer — it strips to a fixed point
    /// before looking up — so it is not a second policy; it is the same policy
    /// with weaker provenance, because a bare `syn::Type` is not proof the model
    /// classified anything.
    ///
    /// Every caller of this is migration debt: the reading was available further
    /// up and was discarded on the way here. The count going to zero is the
    /// measure, the way `flat_input.rs` went 18 → 0 on the spelling census.
    pub(crate) fn declaration_of_spelling(&self, ty: &syn::Type) -> Option<&TypeConfig> {
        let mut stripped = ty.clone();
        while let Some((_, inner)) = crate::api::core::flat::peel_transparent(&stripped) {
            stripped = inner;
        }
        self.map.get(&TypeKey::from_type(&stripped))
    }

    /// The declaration under a declared **name**.
    ///
    /// Safe by construction and not confusable with [`Self::declaration`]: an
    /// ident cannot carry a wrapper, and the compiler will not let one stand in
    /// for the other.
    pub(crate) fn declaration_of_name(&self, ident: &syn::Ident) -> Option<&TypeConfig> {
        self.map.get(&TypeKey::from_ident(ident))
    }

    /// Every declaration, for the emitters that walk the whole table rather
    /// than asking about one type. Order is the caller's business — the
    /// emitters sort by key for determinism.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&TypeKey, &TypeConfig)> {
        self.map.iter()
    }

    /// The keys alone, for a walk that only needs identities.
    pub(crate) fn keys(&self) -> impl Iterator<Item = &TypeKey> {
        self.map.keys()
    }

    /// The declaration stored under an identity this table already handed out.
    ///
    /// For a caller that took a key from [`Self::keys`] or [`Self::iter`] and is
    /// looking the entry back up — the key came from here, so it is already
    /// stripped. **Not** a door for a key built from a spelling: there is no
    /// spelling to strip at this point, which is what makes it safe.
    pub(crate) fn by_declared_key(&self, key: &TypeKey) -> Option<&TypeConfig> {
        self.map.get(key)
    }

    /// Whether this identity is declared. Takes a key the table already handed
    /// out, for the same reason [`Self::by_declared_key`] does.
    pub(crate) fn contains_declared_key(&self, key: &TypeKey) -> bool {
        self.map.contains_key(key)
    }

    /// Mutable access to an entry this table already holds, for the acceptors
    /// that fold cross-kind options in after `register_class` created it.
    pub(crate) fn declared_mut(&mut self, key: &TypeKey) -> Option<&mut TypeConfig> {
        self.map.get_mut(key)
    }

    /// The writer's door — `register_class` only.
    pub(crate) fn entry(
        &mut self,
        key: TypeKey,
    ) -> std::collections::hash_map::Entry<'_, TypeKey, TypeConfig> {
        self.map.entry(key)
    }
}
