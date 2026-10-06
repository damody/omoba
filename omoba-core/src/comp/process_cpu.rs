//! Process-aggregate CPU time in nanoseconds.
//!
//! On Windows, [`process_cpu_ns`] calls `kernel32` `GetProcessTimes` with the
//! `GetCurrentProcess` pseudo-handle. Kernel and user `FILETIME` values are
//! 100-nanosecond counters. They are summed as `u64` and converted with
//! `u128 * 100`. The pseudo-handle is not a real handle and is not closed.
//!
//! The sample is the process total across every thread. It may be larger
//! than wall-clock time when several threads run at once. This module does
//! not subtract CPU time from wall time and does not report wait time.
//!
//! `GetProcessTimes` returning a zero `BOOL`, or a kernel+user sum that does
//! not fit in `u64`, yields `None`. Every other platform returns `None` and
//! has no fallback clock.

/// Sum two Windows 100-nanosecond counters and scale to nanoseconds.
#[cfg(any(windows, test))]
fn filetime_pair_to_ns(kernel: u64, user: u64) -> Option<u128> {
    let ticks = kernel.checked_add(user)?;
    Some(u128::from(ticks) * 100)
}

/// Process CPU time in nanoseconds, or `None` when it cannot be read.
///
/// Windows: kernel time plus user time for the current process. Other
/// platforms always return `None`.
pub fn process_cpu_ns() -> Option<u128> {
    #[cfg(windows)]
    {
        return win32::query_process_cpu_ns();
    }
    #[cfg(not(windows))]
    {
        None
    }
}

/// Nanoseconds of process CPU time from `start` to `end`.
///
/// `None` if either sample is missing, or if `end < start` (the counter
/// moved backwards). Equal samples return `Some(0)`.
pub fn process_cpu_delta(start: Option<u128>, end: Option<u128>) -> Option<u128> {
    end?.checked_sub(start?)
}

/// Minimal `kernel32` FFI. Compiled only for Windows.
#[cfg(windows)]
mod win32 {
    use std::ffi::c_void;
    use std::mem::MaybeUninit;

    type Bool = i32;
    type Dword = u32;
    type Handle = *mut c_void;

    /// Windows `FILETIME`: 100-nanosecond ticks split into low and high dwords.
    #[repr(C)]
    struct FileTime {
        dw_low_date_time: Dword,
        dw_high_date_time: Dword,
    }

    impl FileTime {
        fn to_u64(self) -> u64 {
            (u64::from(self.dw_high_date_time) << 32) | u64::from(self.dw_low_date_time)
        }
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        #[link_name = "GetCurrentProcess"]
        fn get_current_process() -> Handle;

        #[link_name = "GetProcessTimes"]
        fn get_process_times(
            process: Handle,
            creation: *mut FileTime,
            exit: *mut FileTime,
            kernel: *mut FileTime,
            user: *mut FileTime,
        ) -> Bool;
    }

    pub(super) fn query_process_cpu_ns() -> Option<u128> {
        // SAFETY: `GetCurrentProcess` returns the pseudo-handle (`-1`).
        // It is valid for `GetProcessTimes` and must not be closed.
        // The four pointers are live stack slots. On a non-zero BOOL the
        // API has written kernel and user before they are read. A zero
        // BOOL is failure; those slots stay unread.
        unsafe {
            let mut creation = MaybeUninit::<FileTime>::uninit();
            let mut exit = MaybeUninit::<FileTime>::uninit();
            let mut kernel = MaybeUninit::<FileTime>::uninit();
            let mut user = MaybeUninit::<FileTime>::uninit();
            let ok = get_process_times(
                get_current_process(),
                creation.as_mut_ptr(),
                exit.as_mut_ptr(),
                kernel.as_mut_ptr(),
                user.as_mut_ptr(),
            );
            if ok == 0 {
                return None;
            }
            let kernel = kernel.assume_init().to_u64();
            let user = user.assume_init().to_u64();
            super::filetime_pair_to_ns(kernel, user)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{process_cpu_delta, process_cpu_ns};

    #[test]
    fn delta_positive() {
        assert_eq!(process_cpu_delta(Some(1_000), Some(1_500)), Some(500));
        assert_eq!(process_cpu_delta(Some(0), Some(100)), Some(100));
    }

    #[test]
    fn delta_zero() {
        assert_eq!(process_cpu_delta(Some(0), Some(0)), Some(0));
        assert_eq!(process_cpu_delta(Some(42), Some(42)), Some(0));
    }

    #[test]
    fn delta_regression_is_none() {
        assert_eq!(process_cpu_delta(Some(10), Some(9)), None);
        assert_eq!(process_cpu_delta(Some(1), Some(0)), None);
    }

    #[test]
    fn delta_missing_is_none() {
        assert_eq!(process_cpu_delta(None, Some(1)), None);
        assert_eq!(process_cpu_delta(Some(1), None), None);
        assert_eq!(process_cpu_delta(None, None), None);
    }

    #[test]
    fn delta_large_counters() {
        let start = u128::MAX - 1_000;
        let end = u128::MAX - 100;
        assert_eq!(process_cpu_delta(Some(start), Some(end)), Some(900));
        assert_eq!(
            process_cpu_delta(Some(u128::from(u64::MAX)), Some(u128::from(u64::MAX) + 100)),
            Some(100)
        );
        assert_eq!(process_cpu_delta(Some(u128::MAX), Some(u128::MAX)), Some(0));
        assert_eq!(
            process_cpu_delta(Some(u128::MAX), Some(u128::MAX - 1)),
            None
        );
    }

    #[test]
    fn filetime_pair_scales_by_100_and_rejects_overflow() {
        assert_eq!(super::filetime_pair_to_ns(1, 2), Some(300));
        assert_eq!(super::filetime_pair_to_ns(0, 0), Some(0));
        assert_eq!(
            super::filetime_pair_to_ns(u64::MAX, 0),
            Some(u128::from(u64::MAX) * 100)
        );
        assert_eq!(super::filetime_pair_to_ns(u64::MAX, 1), None);
    }

    /// Read-only query of this process. Does not change priority, handles, or threads.
    #[cfg(windows)]
    #[test]
    fn current_process_query_readonly_sanity() {
        let first = process_cpu_ns().expect("GetProcessTimes on the current process");
        // A successful query may report zero at counter granularity;
        // availability must not be inferred from a positive CPU value.
        assert_eq!(first % 100, 0, "FILETIME scale is 100 ns");

        let second = process_cpu_ns().expect("second GetProcessTimes read");
        assert_eq!(second % 100, 0);
        assert!(second >= first, "process CPU counter moved backwards");
        assert_eq!(
            process_cpu_delta(Some(first), Some(second)),
            Some(second - first)
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn process_cpu_ns_unsupported_platform_is_none() {
        assert_eq!(process_cpu_ns(), None);
    }
}
