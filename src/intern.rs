//! String interning for reduced memory usage
//!
//! This module provides string interning to reduce memory overhead when
//! tracking many tasks with similar or repeated names.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// A handle to an interned string
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternedString(u32);

impl InternedString {
    /// Get the string value from the global interner
    #[must_use]
    pub fn as_str(&self) -> String {
        StringInterner::global().resolve(*self)
    }
}

impl std::fmt::Display for InternedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// String interner for deduplicating strings
pub struct StringInterner {
    /// Map from string to intern ID
    string_to_id: RwLock<HashMap<Arc<str>, u32>>,
    /// Map from intern ID to string
    id_to_string: RwLock<HashMap<u32, Arc<str>>>,
    /// Counter for generating unique IDs
    next_id: AtomicU32,
}

/// Global string interner instance
static GLOBAL_INTERNER: once_cell::sync::Lazy<StringInterner> =
    once_cell::sync::Lazy::new(StringInterner::new);

impl StringInterner {
    /// Create a new string interner
    #[must_use]
    pub fn new() -> Self {
        Self {
            string_to_id: RwLock::new(HashMap::new()),
            id_to_string: RwLock::new(HashMap::new()),
            next_id: AtomicU32::new(1),
        }
    }

    /// Get the global string interner
    #[must_use]
    pub fn global() -> &'static Self {
        &GLOBAL_INTERNER
    }

    /// Intern a string, returning a handle
    pub fn intern(&self, s: &str) -> InternedString {
        // Fast path: check if already interned (read lock)
        {
            let string_to_id = self.string_to_id.read();
            if let Some(&id) = string_to_id.get(s) {
                return InternedString(id);
            }
        }

        // Slow path: insert new string (write lock)
        let mut string_to_id = self.string_to_id.write();
        let mut id_to_string = self.id_to_string.write();

        // Double-check after acquiring write lock
        if let Some(&id) = string_to_id.get(s) {
            return InternedString(id);
        }

        // Create new interned string
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let arc_str: Arc<str> = Arc::from(s);

        string_to_id.insert(arc_str.clone(), id);
        id_to_string.insert(id, arc_str);

        InternedString(id)
    }

    /// Resolve an interned string handle to its value
    #[must_use]
    pub fn resolve(&self, handle: InternedString) -> String {
        self.id_to_string
            .read()
            .get(&handle.0)
            .map(|s| s.to_string())
            .unwrap_or_else(|| "<unknown>".to_string())
    }

    /// Get the number of unique strings interned
    #[must_use]
    pub fn len(&self) -> usize {
        self.string_to_id.read().len()
    }

    /// Check if the interner is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get approximate memory usage in bytes
    #[must_use]
    pub fn memory_usage(&self) -> usize {
        let string_to_id = self.string_to_id.read();
        let id_to_string = self.id_to_string.read();

        // Estimate: HashMap overhead + string bytes + Arc overhead
        let map_overhead = (string_to_id.capacity() + id_to_string.capacity())
            * (std::mem::size_of::<u32>() + std::mem::size_of::<Arc<str>>());

        let string_bytes: usize = id_to_string.values().map(|s| s.len()).sum();

        map_overhead + string_bytes
    }

    /// Clear all interned strings (use with caution!)
    pub fn clear(&self) {
        self.string_to_id.write().clear();
        self.id_to_string.write().clear();
        self.next_id.store(1, Ordering::Relaxed);
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to intern a string using the global interner
#[must_use]
pub fn intern(s: &str) -> InternedString {
    StringInterner::global().intern(s)
}

/// Helper function to resolve an interned string
#[must_use]
pub fn resolve(handle: InternedString) -> String {
    StringInterner::global().resolve(handle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intern_deduplication() {
        let interner = StringInterner::new();

        let s1 = interner.intern("hello");
        let s2 = interner.intern("hello");
        let s3 = interner.intern("world");

        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
        assert_eq!(interner.len(), 2);
    }

    #[test]
    fn test_resolve() {
        let interner = StringInterner::new();

        let handle = interner.intern("test_string");
        assert_eq!(interner.resolve(handle), "test_string");
    }

    #[test]
    fn test_global_interner() {
        let s1 = intern("global_test");
        let s2 = intern("global_test");
        assert_eq!(s1, s2);
    }
}
