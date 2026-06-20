#[macro_export]
macro_rules! info {
	($($arg:tt)*) => {
		{
			let (f, l, c) = (file!(), line!(), column!());
			let m = ::std::format!("[{f}:{l}:{c}]: {}", ::std::format!($($arg)*).as_str());
			#[cfg(feature = "backtrace")]
			let m = ::std::format!("{m}\n=== BACKTRACE ===\n{:?}", backtrace::Backtrace::new());
			#[cfg(feature = "stderr")]
			eprintln!("{m}");
			#[cfg(feature = "stdout")]
			println!("{m}");
			#[cfg(not(test))]
			#[cfg(feature = "godot")]
			$crate::godot::print(m.as_str());
			let _ = m;
		}
	};
}

#[macro_export]
macro_rules! warn {
	($($arg:tt)*) => {
		{
			let (f, l, c) = (file!(), line!(), column!());
			let m = ::std::format!("[{f}:{l}:{c}]: {}", ::std::format!($($arg)*).as_str());
			#[cfg(feature = "backtrace")]
			let m = ::std::format!("{m}\n=== BACKTRACE ===\n{:?}", backtrace::Backtrace::new());
			#[cfg(feature = "stderr")]
			eprintln!("{m}");
			#[cfg(feature = "stdout")]
			println!("{m}");
			#[cfg(not(test))]
			#[cfg(feature = "godot")]
			$crate::godot::push_warning(m.as_str());
			#[cfg(not(test))]
			#[cfg(feature = "godot")]
			$crate::godot::print(m.as_str());
			let _ = m;
		}
	};
}

#[macro_export]
macro_rules! error {
	($($arg:tt)*) => {
		{
			let (f, l, c) = (file!(), line!(), column!());
			let m = ::std::format!("[{f}:{l}:{c}]: {}", ::std::format!($($arg)*).as_str());
			#[cfg(feature = "backtrace")]
			let m = ::std::format!("{m}\n=== BACKTRACE ===\n{:?}", backtrace::Backtrace::new());
			#[cfg(feature = "stderr")]
			eprintln!("{m}");
			#[cfg(feature = "stdout")]
			println!("{m}");
			#[cfg(not(test))]
			#[cfg(feature = "godot")]
			$crate::godot::push_error(m.as_str());
			#[cfg(not(test))]
			#[cfg(feature = "godot")]
			$crate::godot::print(m.as_str());
			let _ = m;
		}
	};
}

#[macro_export]
macro_rules! fatal {
	($($arg:tt)*) => {
		{
			let (f, l, c) = (file!(), line!(), column!());
			let m = ::std::format!("[{f}:{l}:{c}]: {}", ::std::format!($($arg)*).as_str());
			#[cfg(feature = "backtrace")]
			let m = ::std::format!("{m}\n=== BACKTRACE ===\n{:?}", backtrace::Backtrace::new());
			#[cfg(feature = "stderr")]
			eprintln!("{m}");
			#[cfg(feature = "stdout")]
			println!("{m}");
			#[cfg(not(test))]
			#[cfg(feature = "godot")]
			$crate::godot::push_error(m.as_str());
			#[cfg(not(test))]
			#[cfg(feature = "godot")]
			$crate::godot::print(m.as_str());
			#[cfg(not(test))]
			#[cfg(feature = "godot")]
			$crate::godot::quit(1);
			::std::panic!("{m}");
		}
	};
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
