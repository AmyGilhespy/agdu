use crate::{error, fatal, info, warn};

#[allow(dead_code)]
fn _test_debugging_macros_compile() {
	info!("info {}", 1);
	warn!("warn {}", 2);
	error!("error {}", 3);
	fatal!("fatal {}", 4);
}
