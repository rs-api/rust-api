//! Request extensions for storing arbitrary data
//!
//! Extensions allow middleware to attach data to requests that can be
//! retrieved by handlers or other middleware downstream.

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// A type map for storing request-scoped data
///
/// Extensions allow you to store arbitrary data that can be accessed
/// by type throughout the request lifecycle.
///
/// # Example
///
/// ```rust,ignore
/// use rust_api::prelude::*;
///
/// #[derive(Clone)]
/// struct User {
///     id: u64,
///     name: String,
/// }
///
/// // In middleware
/// let user = User { id: 1, name: "Alice".into() };
/// req.extensions_mut().insert(user);
///
/// // In handler
/// if let Some(user) = req.extensions().get::<User>() {
///     println!("User: {}", user.name);
/// }
/// ```
#[derive(Default)]
pub struct Extensions {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Extensions {
    /// Create a new empty Extensions
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Insert a value into the extensions
    ///
    /// If a value of this type already exists, it will be replaced and returned.
    pub fn insert<T: Send + Sync + 'static>(&mut self, value: T) -> Option<T> {
        self.map
            .insert(TypeId::of::<T>(), Box::new(value))
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    /// Get a reference to a value in the extensions
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Get a mutable reference to a value in the extensions
    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.map
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    /// Remove a value from the extensions
    pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.map
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    /// Check if a value of type T exists in the extensions
    pub fn contains<T: Send + Sync + 'static>(&self) -> bool {
        self.map.contains_key(&TypeId::of::<T>())
    }

    /// Clear all extensions
    pub fn clear(&mut self) {
        self.map.clear();
    }
}

impl std::fmt::Debug for Extensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Extensions")
            .field("count", &self.map.len())
            .finish()
    }
}
