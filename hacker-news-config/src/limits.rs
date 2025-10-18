//! Unix number of open files limits
use libc::{getrlimit, rlimit, setrlimit, RLIMIT_NOFILE};
use log::{error, info};

/// Increase the number open files limit on unix.
pub fn check_nofiles_limit() {
    const DESIRED_LIMIT: u64 = 10_240;

    let mut rlim = rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };

    unsafe {
        if getrlimit(RLIMIT_NOFILE, &mut rlim) != 0 {
            let errno = std::io::Error::last_os_error();
            error!("Could not get open files limit: {errno}");
            return;
        }
    }

    info!(
        "Current open file limits: current {}, max {}",
        rlim.rlim_cur, rlim.rlim_max
    );

    if rlim.rlim_cur < DESIRED_LIMIT {
        rlim.rlim_cur = DESIRED_LIMIT;
        rlim.rlim_max = DESIRED_LIMIT;

        unsafe {
            if setrlimit(RLIMIT_NOFILE, &rlim) != 0 {
                let errno = std::io::Error::last_os_error();
                error!("Could not set open files limit: {errno}");
                return;
            }
        }
        info!("Increased open file limit to {DESIRED_LIMIT}");
    }
}
