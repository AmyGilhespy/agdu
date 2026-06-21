#[macro_export]
macro_rules! info {
	($($arg:tt)*) => {
		emit_info(file!(), line!(), column!(), ::std::format!($($arg)*));
	};
}

#[macro_export]
macro_rules! warn {
	($($arg:tt)*) => {
		emit_warn(file!(), line!(), column!(), ::std::format!($($arg)*));
	};
}

#[macro_export]
macro_rules! error {
	($($arg:tt)*) => {
		emit_error(file!(), line!(), column!(), ::std::format!($($arg)*));
	};
}

#[macro_export]
macro_rules! fatal {
	($($arg:tt)*) => {
		emit_fatal(file!(), line!(), column!(), ::std::format!($($arg)*));
		::std::unreachable!();
	};
}

#[allow(clippy::needless_pass_by_value)]
pub fn emit_info(file: &str, line: u32, column: u32, message: String) {
	let message = ::std::format!("[{file}:{line}:{column}]: {message}");
	#[cfg(feature = "stderr")]
	eprintln!("{message}");
	#[cfg(feature = "stdout")]
	println!("{message}");
	#[cfg(not(test))]
	#[cfg(feature = "godot")]
	crate::godot::print(message.as_str());
	let _ = message;
}

#[allow(clippy::needless_pass_by_value)]
pub fn emit_warn(file: &str, line: u32, column: u32, message: String) {
	let message = ::std::format!("[{file}:{line}:{column}]: {message}");
	#[cfg(feature = "stderr")]
	eprintln!("{message}");
	#[cfg(feature = "stdout")]
	println!("{message}");
	#[cfg(not(test))]
	#[cfg(feature = "godot")]
	crate::godot::push_warning(message.as_str());
	#[cfg(not(test))]
	#[cfg(feature = "godot")]
	crate::godot::print(message.as_str());
	let _ = message;
}

#[allow(clippy::needless_pass_by_value)]
pub fn emit_error(file: &str, line: u32, column: u32, message: String) {
	let message = ::std::format!("[{file}:{line}:{column}]: {message}");
	#[cfg(feature = "backtrace")]
	let message = ::std::format!(
		"{message}\n=== BACKTRACE ===\n{:?}",
		backtrace::Backtrace::new()
	);
	#[cfg(feature = "stderr")]
	eprintln!("{message}");
	#[cfg(feature = "stdout")]
	println!("{message}");
	#[cfg(not(test))]
	#[cfg(feature = "godot")]
	crate::godot::push_error(message.as_str());
	#[cfg(not(test))]
	#[cfg(feature = "godot")]
	crate::godot::print(message.as_str());
	let _ = message;
}

/// # Panics
#[allow(clippy::needless_pass_by_value)]
pub fn emit_fatal(file: &str, line: u32, column: u32, message: String) {
	let message = ::std::format!("[{file}:{line}:{column}]: {message}");
	#[cfg(feature = "backtrace")]
	let message = ::std::format!(
		"{message}\n=== BACKTRACE ===\n{:?}",
		backtrace::Backtrace::new()
	);
	#[cfg(feature = "stderr")]
	eprintln!("{message}");
	#[cfg(feature = "stdout")]
	println!("{message}");
	#[cfg(not(test))]
	#[cfg(feature = "godot")]
	crate::godot::push_error(message.as_str());
	#[cfg(not(test))]
	#[cfg(feature = "godot")]
	crate::godot::print(message.as_str());
	#[cfg(not(test))]
	#[cfg(feature = "godot")]
	crate::godot::quit(1);
	::std::panic!("{message}");
}

#[cfg(test)]
mod tests {
	#[allow(unused_imports)]
	use super::*;

	#[test]
	fn test_debugging_macros_except_fatal() {
		info!("info {}", 1);
		warn!("warn {}", 2);
		error!("error {}", 3);
	}

	#[test]
	#[should_panic]
	fn test_fatal() {
		fatal!("fatal {}", 4);
	}
}
