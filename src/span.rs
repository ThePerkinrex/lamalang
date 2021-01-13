use std::{
	fmt::{Debug, Display},
	path::PathBuf,
};

use crate::{
	error::{Error, ErrorCode},
	fs::File,
};

#[derive(Clone)]
pub struct Span<T> {
	range: RangedPosition,
	file: File,
	content: T,
}

#[derive(Debug, Clone, Copy)]
pub struct RangedPosition {
	start: (usize, usize),
	end: (usize, usize),
}

impl From<&pest::Span<'_>> for RangedPosition {
	fn from(span: &pest::Span<'_>) -> Self {
		Self {
			start: span.start_pos().line_col(),
			end: span.end_pos().line_col(),
		}
	}
}

impl From<pest::Span<'_>> for RangedPosition {
	fn from(span: pest::Span<'_>) -> Self {
		(&span).into()
	}
}

impl From<&Self> for RangedPosition {
	fn from(s: &Self) -> Self {
		*s
	}
}

impl<T> Span<T> {
	pub fn new(span: pest::Span, file: File, content: T) -> Self {
		Self {
			content,
			file,
			range: span.into(),
		}
	}

	pub fn new_from_inner<'a, S: 'a>(inner: &'a [S], file: File, content: T) -> Self
	where
		RangedPosition: From<&'a S>,
	{
		let start = inner
			.iter()
			.map::<RangedPosition, _>(|x| x.into())
			.map(|s| s.start)
			.fold(None, |a, (current_line, current_col)| {
				if let Some((smallest_line, smallest_col)) = a {
					if smallest_line < current_line {
						Some((smallest_line, smallest_col))
					} else if smallest_line > current_line {
						Some((current_line, current_col))
					} else {
						if smallest_col < current_col {
							Some((smallest_line, smallest_col))
						} else {
							Some((current_line, current_col))
						}
					}
				} else {
					Some((current_line, current_col))
				}
			})
			.expect("Expected at least 1 span");
		let end = inner
			.iter()
			.map::<RangedPosition, _>(|x| x.into())
			.map(|s| s.end)
			.fold(None, |a, (current_line, current_col)| {
				if let Some((smallest_line, smallest_col)) = a {
					if smallest_line > current_line {
						Some((smallest_line, smallest_col))
					} else if smallest_line < current_line {
						Some((current_line, current_col))
					} else {
						if smallest_col > current_col {
							Some((smallest_line, smallest_col))
						} else {
							Some((current_line, current_col))
						}
					}
				} else {
					Some((current_line, current_col))
				}
			})
			.unwrap(); // If there was only 1 span, the previous except caught it

		Self {
			range: RangedPosition { start, end },
			file,
			content,
		}
	}

	pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Span<U> {
		Span {
			content: f(self.content),
			range: self.range,
			file: self.file,
		}
	}

	pub fn as_range(&self) -> RangedPosition {
		self.range
	}

	pub fn as_error<S: ToString>(&self, error_code: ErrorCode, message: S) -> Error {
		Error::new(
			error_code,
			Span::<()> {
				content: (),
				range: self.range,
				file: self.file.clone(),
			},
			message.to_string(),
		)
	}

	pub fn into_error<S: ToString>(self, error_code: ErrorCode, message: S) -> Error {
		Error::new(error_code, self.map(|_| ()), message.to_string())
	}
}

impl<T> std::ops::Deref for Span<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.content
	}
}

impl<T: Debug> Debug for Span<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Debug::fmt(&self.content, f)
	}
}

impl<T: Display> Display for Span<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Display::fmt(&self.content, f)
	}
}

pub type BoxedSpan<T> = Span<Box<T>>;

impl<T> BoxedSpan<T> {
	pub fn boxed(span: pest::Span, file: File, content: T) -> Self {
		Self::new(span, file, Box::new(content))
	}

	pub fn boxed_from_inner<'a, S: 'a>(inner: &'a [S], file: File, content: T) -> Self
	where
		RangedPosition: From<&'a S>,
	{
		Self::new_from_inner(inner, file, Box::new(content))
	}
}
