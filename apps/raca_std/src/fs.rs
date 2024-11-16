use core::fmt;

use alloc::sync::Arc;
use spin::RwLock;
use syscall_macro::syscall;

use crate::path::Path;

#[repr(u8)]
pub enum OpenMode {
    Read = 0,
    Write = 1,
    ReadWrite = 2,
}

struct FileDescriptorInner(usize,bool);

impl FileDescriptorInner {
    /// This opens a file at `path` with mode `open_mode` \
    /// The file will be closed when the file descriptor is dropped.
    pub fn open(path: Path, open_mode: OpenMode) -> Result<Self, ()> {
        const OPEN_SYSCALL_ID: u64 = 1;

        let (ptr,len) = path.to_os_union();

        let fd = syscall!(
            OPEN_SYSCALL_ID,
            ptr,
            len,
            open_mode as usize,
        );
        if fd < 0 {
            Err(())
        } else {
            Ok(Self(fd as usize, false))
        }
    }

    /// Read something to the buffer.
    pub fn read(&self, buffer: &mut [u8]) -> isize {
        assert_ne!(self.1, true, "This File Descriptor had been closed!");

        const READ_SYSCALL_ID: u64 = 3;
        syscall!(
            READ_SYSCALL_ID,
            self.0,
            buffer.as_ptr() as usize,
            buffer.len(),
        )
    }

    /// This function will read until the buffer is full.
    pub fn read_exact(&self, buffer: &mut [u8]) {
        let mut readed = 0;
        while readed < buffer.len() {
            let read_size = self.read(&mut buffer[readed..]);

            if read_size < 0 {
                continue;
            }

            readed += read_size as usize;
        }
    }

    /// Write something to the file.
    pub fn write(&self, buffer: &[u8]) -> isize {
        assert_ne!(self.1, true, "This File Descriptor had been closed!");

        const WRITE_SYSCALL_ID: u64 = 4;
        syscall!(
            WRITE_SYSCALL_ID,
            self.0,
            buffer.as_ptr() as usize,
            buffer.len(),
        )
    }

    /// Seek to the specified position.
    pub fn seek(&self, offset: usize) -> isize {
        assert_ne!(self.1, true, "This File Descriptor had been closed!");

        const LSEEK_SYSCALL_ID: u64 = 5;
        syscall!(LSEEK_SYSCALL_ID, self.0, offset)
    }

    /// Get the size of the file.
    pub fn size(&self) -> isize {
        assert_ne!(self.1, true, "This File Descriptor had been closed!");

        const FSIZE_SYSCALL_ID: u64 = 7;
        syscall!(FSIZE_SYSCALL_ID, self.0,)
    }

    pub(self) fn close(&mut self) {
        self.1 = true;

        const CLOSE_SYSCALL_ID: u64 = 6;
        syscall!(CLOSE_SYSCALL_ID, self.0,);
    }
}

impl Drop for FileDescriptorInner {
    fn drop(&mut self) {
        self.close();
    }
}

#[derive(Clone)]
pub struct File(Arc<RwLock<FileDescriptorInner>>);

impl File {
    pub unsafe fn from_raw_fd(fd: usize) -> Self {
        Self(Arc::new(RwLock::new(FileDescriptorInner(fd, false))))
    }

    pub fn open(path: Path, open_mode: OpenMode) -> Result<Self,()> {
        Ok(Self(Arc::new(RwLock::new(FileDescriptorInner::open(path, open_mode)?))))
    }

    pub fn read(&self, buf: &mut [u8]) -> isize {
        self.0.read().read(buf)
    }

    pub fn read_exact(&self, buf: &mut [u8]) {
        self.0.read().read_exact(buf);
    }

    pub fn write(&self, buf: &[u8]) -> isize {
        self.0.read().write(buf)
    }

    pub fn seek(&self, offset: usize) -> isize {
        self.0.read().seek(offset)
    }

    pub fn size(&self) -> isize {
        self.0.read().size()
    }

    pub fn close(&mut self) {
        self.0.write().close();
    }
}

impl fmt::Write for File {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}


