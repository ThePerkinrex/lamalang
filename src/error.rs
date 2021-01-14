use std::fmt::Display;

use crate::span::Span;

#[repr(i32)]
#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
	// Error
	ModuleNotFoundError,
	NoMainError,
	// Warn
	WarnTest,
	// Info
	InfoTest,
}

impl ErrorCode {
	fn get_kind(&self) -> ErrorKind {
		use ErrorCode::*;
		match self {
			ModuleNotFoundError => ErrorKind::Error,
			NoMainError => ErrorKind::Error,
			WarnTest => ErrorKind::Warn,
			InfoTest => ErrorKind::Info,
		}
	}
}

impl Display for ErrorCode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}[{}]",
			match self.get_kind() {
				ErrorKind::Error => "error",
				ErrorKind::Warn => "warning",
				ErrorKind::Info => "info",
			},
			*self as usize
		)
	}
}

enum ErrorKind {
	Info,
	Warn,
	Error,
}

pub struct Error {
	code: ErrorCode,
	span: Span<()>,
	message: String,
}

impl Error {
	pub fn new(code: ErrorCode, span: Span<()>, message: String) -> Self {
		Self {
			code,
			span,
			message,
		}
	}

	pub fn display(self) -> Result<(), Self> {
		println!("{}: {}", self.code, self.message);

		match self.code.get_kind() {
			ErrorKind::Error => Err(self),
			_ => Ok(()),
		}
	}

	pub fn get_return_code(&self) -> i32 {
		self.code as _
	}
}

pub struct NonLocatedError {
	code: ErrorCode,
	message: String,
}

impl NonLocatedError {
	pub fn new(code: ErrorCode, message: String) -> Self {
		Self { code, message }
	}

	pub fn display(self) -> Result<(), Self> {
		println!("{}: {}", self.code, self.message);

		match self.code.get_kind() {
			ErrorKind::Error => Err(self),
			_ => Ok(()),
		}
	}

	pub fn get_return_code(&self) -> i32 {
		self.code as _
	}
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct ReturnValue {
	pub value: i32,
}

impl From<Error> for ReturnValue {
	fn from(e: Error) -> Self {
		Self {
			value: e.get_return_code(),
		}
	}
}

impl From<NonLocatedError> for ReturnValue {
	fn from(e: NonLocatedError) -> Self {
		Self {
			value: e.get_return_code(),
		}
	}
}

pub type Return<T> = Result<T, ReturnValue>;
