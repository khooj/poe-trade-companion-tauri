use crate::model;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    sync::{Arc, Mutex},
};
use log::debug;

pub trait Len {
    fn len(&self) -> std::io::Result<u64>;
}

pub trait FileLineReaderSource: Read + Seek + Len {}

impl Len for File {
    fn len(&self) -> std::io::Result<u64> {
        let m = File::metadata(self)?;
        Ok(m.len())
    }
}

impl FileLineReaderSource for File {}

pub struct FileLineReader<F> {
    byte_count: u64,
    sock: F,
    model: Arc<Mutex<model::Model>>,
}

impl<F> FileLineReader<F> {
    pub fn with_file(
        model: Arc<Mutex<model::Model>>,
        fp: &str,
    ) -> Result<FileLineReader<File>, anyhow::Error> {
        FileLineReader::new(model, File::open(fp)?)
    }
}

impl<F: FileLineReaderSource> FileLineReader<F> {
    pub fn new(model: Arc<Mutex<model::Model>>, mut fp: F) -> Result<Self, anyhow::Error> {
        fp.seek(SeekFrom::End(0))?;
        let size = fp.len()?;

        Ok(FileLineReader {
            byte_count: size,
            sock: fp,
            model,
        })
    }

    pub fn process_new_content(&mut self) -> Result<(), anyhow::Error> {
        let new_size = self.sock.len()?;
        if new_size <= self.byte_count {
            // probably file truncated or nothing new added, just update byte_count and wait for new call
            self.byte_count = new_size;
            debug!("no new content in source");
            return Ok(());
        }
        self.byte_count = new_size;

        let mut contents = String::new();
        self.sock.read_to_string(&mut contents)?;
        debug!("read {} data to process", contents.len());
        let mut lock = self.model.lock().unwrap();
        for l in contents.lines() {
            // add log
            let r = lock.try_add(l);
            debug!("result processing line: {} {:?}", l, r);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::Callable;
    use std::io::Cursor;
    use std::io::Write;

    #[derive(Clone)]
    struct CursorMut {
        m: Arc<Mutex<Cursor<Vec<u8>>>>,
    }

    impl CursorMut {
        fn new() -> Self {
            CursorMut {
                m: Arc::new(Mutex::new(Cursor::new(vec![]))),
            }
        }
    }

    impl Read for CursorMut {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.m.lock().unwrap().read(buf)
        }
    }

    impl Seek for CursorMut {
        fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
            self.m.lock().unwrap().seek(pos)
        }
    }

    impl std::io::Write for CursorMut {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.m.lock().unwrap().write(buf)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl Len for CursorMut {
        fn len(&self) -> std::io::Result<u64> {
            let mut m = self.m.lock().unwrap();
            let cp = m.position();
            m.seek(SeekFrom::End(0))?;
            let size = m.position() - cp;
            m.seek(SeekFrom::Start(cp))?;
            Ok(size)
        }
    }

    impl FileLineReaderSource for CursorMut {}

    struct Pos {
        read: u64,
        write: u64,
    }

    #[derive(Clone)]
    struct DoubleBuffer {
        b: CursorMut,
        pos: Arc<Mutex<Pos>>,
    }

    impl DoubleBuffer {
        fn new() -> Self {
            DoubleBuffer {
                b: CursorMut::new(),
                pos: Arc::new(Mutex::new(Pos { read: 0, write: 0 })),
            }
        }
    }

    impl Read for DoubleBuffer {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let mut m = self.pos.lock().unwrap();
            self.b.seek(SeekFrom::Start(m.read))?;
            let s = self.b.read(buf)?;
            m.read += s as u64;
            Ok(s)
        }
    }

    impl Write for DoubleBuffer {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let mut m = self.pos.lock().unwrap();
            self.b.seek(SeekFrom::Start(m.write))?;
            let s = self.b.write(buf)?;
            m.write += s as u64;
            Ok(s)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl Seek for DoubleBuffer {
        fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
            let s = self.b.seek(pos)?;
            self.pos.lock().unwrap().read = s;
            Ok(s)
        }
    }

    impl Len for DoubleBuffer {
        fn len(&self) -> std::io::Result<u64> {
            Ok(self.pos.lock().unwrap().write)
        }
    }

    impl FileLineReaderSource for DoubleBuffer {}

    #[test]
    fn double_buffer_usage() {
        let mut db = DoubleBuffer::new();
        db.write("buf".as_bytes()).unwrap();
        let mut b = [0; 3];
        db.read(&mut b).unwrap();
        assert_eq!(b, "buf".as_bytes());
    }

    const INCOMING_MSG: &'static [u8] = r#"2023/10/13 01:54:50 1054470421 cffb0719 [INFO Client 30680] @From SambaLe: Hi, I would like to buy your The Pandemonius, Jade Amulet listed for 4 divine in Ancestor (stash tab "pub"; position: left 11, top 1)"#.as_bytes();

    #[test]
    fn basic_usage() {
        let mut buf = DoubleBuffer::new();
        let model = Arc::new(Mutex::new(model::Model::new()));
        let mut rdr = FileLineReader::new(Arc::clone(&model), buf.clone()).unwrap();
        let clb = Callable::new();

        {
            let clb = clb.clone();
            model.lock().unwrap().incoming_subscribe(move |_ig| {
                clb.call();
            });
        }
        buf.write(INCOMING_MSG).unwrap();
        buf.write(b"\n").unwrap();
        buf.write(INCOMING_MSG).unwrap();
        rdr.process_new_content().unwrap();

        assert_eq!(clb.count(), 2);

        buf.write(b"\n").unwrap();
        buf.write(INCOMING_MSG).unwrap();
        rdr.process_new_content().unwrap();

        assert_eq!(clb.count(), 3);
    }
}
