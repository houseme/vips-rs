//! Cache and memory traces: Encapsulating 'vips_cache_*' and 'vips_tracked_*'
//!
//! Reference:
//! - <https://www.libvips.org/API/current/libvips-cache.html>
//! - <https://www.libvips.org/API/current/libvips-memory.html>

/// Sets the maximum number of compute nodes that can be retained in the cache
///
/// # Arguments
/// * `n` - The maximum number of operations to retain
///
pub fn set_max_operations(n: i32) {
    unsafe { vips_sys::vips_cache_set_max(n) }
}

/// Set the maximum memory allowed in bytes for the cache
///
/// # Arguments
/// * `n` - The maximum memory in bytes
///
pub fn set_max_mem_bytes(n: usize) {
    unsafe { vips_sys::vips_cache_set_max_mem(n) }
}

/// Set the maximum number of files allowed to open at the same time
///
/// # Arguments
/// * `n` - The maximum number of open files
///
pub fn set_max_files(n: i32) {
    unsafe { vips_sys::vips_cache_set_max_files(n) }
}

/// Gets the number of compute nodes in the current cache
///
/// # Returns
/// The current size of the operations cache
///
pub fn size_operations() -> i32 {
    unsafe { vips_sys::vips_cache_get_size() }
}

/// Gets the maximum number of compute nodes configured
///
/// # Returns
/// The maximum size of the operations cache
///
pub fn max_operations() -> i32 {
    unsafe { vips_sys::vips_cache_get_max() }
}

/// Get the maximum cache memory (bytes) configured
///
/// # Returns
/// The maximum memory in bytes for the cache
///
pub fn max_mem_bytes() -> usize {
    unsafe { vips_sys::vips_cache_get_max_mem() }
}

/// Get the maximum number of files configured
///
/// # Returns
/// The maximum number of open files allowed
///
pub fn max_files() -> i32 {
    unsafe { vips_sys::vips_cache_get_max_files() }
}

/// Currently tracked memory (bytes)
///
/// # Returns
/// The current memory usage in bytes tracked by libvips
///
pub fn tracked_mem_bytes() -> usize {
    unsafe { vips_sys::vips_tracked_get_mem() }
}

/// Historical Peak Memory Usage (bytes)
///
/// # Returns
/// The highwater mark of memory usage in bytes tracked by libvips
///
pub fn tracked_mem_highwater_bytes() -> usize {
    unsafe { vips_sys::vips_tracked_get_mem_highwater() }
}

/// Print the diagnostic information for each cache hit/build (1 on, 0 off)
///
/// # Arguments
/// * `trace` - Enable or disable cache tracing
///
pub fn set_trace(trace: bool) {
    unsafe { vips_sys::vips_cache_set_trace(if trace { 1 } else { 0 }) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init;
    use std::sync::Mutex;

    /// libvips global state is process-wide; serialize tests that touch it.
    fn vips_test_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn version_works() {
        let _guard = vips_test_lock();
        let v = crate::version::version();
        assert!(v.0 >= 8);
        assert!(crate::version::version_string().contains("8."));
    }

    #[test]
    fn init_is_idempotent() {
        let _guard = vips_test_lock();
        init::init(Some("test-app")).unwrap();
        init::init(Some("test-app")).unwrap();
        assert!(init::is_initialized());
    }

    #[test]
    fn cache_controls() {
        let _guard = vips_test_lock();
        init::init(Some("cache-test")).unwrap();
        set_max_operations(256);
        set_max_mem_bytes(64 * 1024 * 1024);
        set_max_files(64);
        assert!(max_operations() >= 1);
        assert!(max_mem_bytes() >= 1);
        assert!(max_files() >= 1);
    }
}
