// In-app console.
//
// The launcher used to toggle a real Windows console window, which has no
// counterpart on macOS or Linux. Redirecting our own stdout and stderr into a ring
// buffer and rendering that in the launcher works identically everywhere - and it
// finally gives release builds somewhere to show their log at all, since those are
// linked as GUI binaries with no console attached.
//
// Whatever stream we started with is kept and written through, so a build launched
// from a terminal still prints where the developer expects.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::sync::OnceLock;

use parking_lot::Mutex;
use serde::Serialize;

/// Enough history to cover a full startup plus a while of running.
const MAX_LINES: usize = 2000;

#[derive(Debug, Serialize)]
pub struct LogChunk {
    pub lines: Vec<String>,
    /// Cursor to pass to the next call.
    pub next: u64,
    /// True when the caller's cursor was so far behind that lines were dropped.
    pub truncated: bool,
}

#[derive(Default)]
struct Buffer {
    lines: VecDeque<String>,
    /// Sequence number of `lines[0]`.
    first: u64,
}

impl Buffer {
    fn push(&mut self, line: String, capacity: usize) {
        self.lines.push_back(line);
        while self.lines.len() > capacity {
            self.lines.pop_front();
            self.first += 1;
        }
    }

    fn read_since(&self, since: u64) -> LogChunk {
        let end = self.first + self.lines.len() as u64;

        if since >= end {
            return LogChunk {
                lines: Vec::new(),
                next: end,
                truncated: false,
            };
        }

        LogChunk {
            lines: self
                .lines
                .iter()
                .skip(since.saturating_sub(self.first) as usize)
                .cloned()
                .collect(),
            next: end,
            truncated: since < self.first,
        }
    }
}

fn buffer() -> &'static Mutex<Buffer> {
    static BUFFER: OnceLock<Mutex<Buffer>> = OnceLock::new();
    BUFFER.get_or_init(|| Mutex::new(Buffer::default()))
}

fn push(line: String) {
    buffer().lock().push(line, MAX_LINES);
}

/// Returns everything recorded after `since`. Callers start at 0 and pass back the
/// `next` cursor.
pub fn read_since(since: u64) -> LogChunk {
    buffer().lock().read_since(since)
}

/// Splits the pipe into lines and keeps writing them through to the original
/// stream. Never prints anything itself - that would feed straight back in.
fn pump(reader: impl std::io::Read + Send + 'static, mut passthrough: Option<std::fs::File>) {
    std::thread::spawn(move || {
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) | Err(_) => return,
                Ok(_) => {}
            }

            if let Some(original) = passthrough.as_mut() {
                let _ = original.write_all(line.as_bytes());
                let _ = original.flush();
            }
            push(line.trim_end_matches(['\r', '\n']).to_string());
        }
    });
}

/// Redirects stdout and stderr into the buffer. Call once, before anything prints.
pub fn capture() {
    static STARTED: OnceLock<()> = OnceLock::new();
    if STARTED.set(()).is_err() {
        return;
    }
    install();
}

#[cfg(unix)]
fn install() {
    use std::os::fd::FromRawFd;

    unsafe {
        let mut fds = [0 as libc::c_int; 2];
        if libc::pipe(fds.as_mut_ptr()) != 0 {
            return;
        }
        let (read, write) = (fds[0], fds[1]);

        // Keep the stream we started with so terminal runs still print.
        let saved = libc::dup(libc::STDOUT_FILENO);
        let passthrough = if saved >= 0 {
            Some(std::fs::File::from_raw_fd(saved))
        } else {
            None
        };

        if libc::dup2(write, libc::STDOUT_FILENO) < 0 || libc::dup2(write, libc::STDERR_FILENO) < 0
        {
            libc::close(read);
            libc::close(write);
            return;
        }
        libc::close(write);

        pump(std::fs::File::from_raw_fd(read), passthrough);
    }
}

#[cfg(windows)]
fn install() {
    use std::os::windows::io::FromRawHandle;
    use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};
    use windows::Win32::System::Console::{
        GetStdHandle, SetStdHandle, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE,
    };
    use windows::Win32::System::Pipes::CreatePipe;

    unsafe {
        let mut read = HANDLE::default();
        let mut write = HANDLE::default();
        if CreatePipe(&mut read, &mut write, None, 0).is_err() {
            return;
        }

        // A release build is linked as a GUI binary, so there is usually no console
        // here and this comes back invalid - which just means nothing to pass to.
        let passthrough = match GetStdHandle(STD_OUTPUT_HANDLE) {
            Ok(handle) if !handle.is_invalid() && handle != INVALID_HANDLE_VALUE => {
                Some(std::fs::File::from_raw_handle(handle.0))
            }
            _ => None,
        };

        if SetStdHandle(STD_OUTPUT_HANDLE, write).is_err()
            || SetStdHandle(STD_ERROR_HANDLE, write).is_err()
        {
            return;
        }

        pump(std::fs::File::from_raw_handle(read.0), passthrough);
    }
}

#[cfg(not(any(unix, windows)))]
fn install() {}

#[cfg(test)]
mod tests {
    use super::*;

    // Exercises Buffer directly; the process-wide one is shared with the real
    // stdout pump, so tests must not race it.
    fn filled(count: usize, capacity: usize) -> Buffer {
        let mut buffer = Buffer::default();
        for index in 0..count {
            buffer.push(index.to_string(), capacity);
        }
        buffer
    }

    #[test]
    fn hands_back_only_what_is_new() {
        let buffer = filled(2, 10);

        let chunk = buffer.read_since(0);
        assert_eq!(chunk.lines, ["0", "1"]);
        assert_eq!(chunk.next, 2);
        assert!(!chunk.truncated);

        let chunk = buffer.read_since(chunk.next);
        assert!(chunk.lines.is_empty());
        assert_eq!(chunk.next, 2);
    }

    #[test]
    fn drops_the_oldest_lines_and_reports_the_gap() {
        let buffer = filled(15, 10);
        let chunk = buffer.read_since(0);

        assert_eq!(chunk.lines.len(), 10);
        assert_eq!(chunk.lines[0], "5");
        assert!(chunk.truncated);
        assert_eq!(chunk.next, 15);
    }

    #[test]
    fn a_cursor_inside_the_window_is_not_truncated() {
        let chunk = filled(10, 10).read_since(7);

        assert_eq!(chunk.lines, ["7", "8", "9"]);
        assert!(!chunk.truncated);
    }
}
