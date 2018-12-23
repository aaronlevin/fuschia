#![feature(const_str_len)]

extern crate env_logger;
extern crate fuse;
extern crate libc;
extern crate time;

use fuse::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request,
};
use libc::ENOENT;
use std::env;
use std::ffi::OsStr;
use time::Timespec;

pub struct GameFile {
    name: String,
    content: String,
}
impl GameFile {
    pub fn new(name: String, content: String) -> GameFile {
        GameFile {
            name: name,
            content: content,
        }
    }
    pub fn content(mut self, content: &str) -> Self {
        self.content = content.to_string();
        self
    }
}

pub struct GameDir {
    name: String,
    files: Vec<GameFile>,
    sub_dirs: Vec<GameDir>,
}
impl GameDir {
    pub fn new(name: String) -> GameDir {
        GameDir {
            name: name,
            files: Vec::new(),
            sub_dirs: Vec::new(),
        }
    }
    pub fn with_file(mut self, file: GameFile) -> Self {
        self.files.push(file);
        self
    }
    pub fn with_dir(mut self, dir: GameDir) -> Self {
        self.sub_dirs.push(dir);
        self
    }
}

pub fn dir(name: &str) -> GameDir {
    GameDir::new(name.to_string())
}
pub fn file(name: &str) -> GameFile {
    GameFile::new(name.to_string(), "".to_string())
}

const TTL: Timespec = Timespec { sec: 1, nsec: 0 }; // 1 second

const CREATE_TIME: Timespec = Timespec {
    sec: 1381237736,
    nsec: 0,
}; // 2013-10-08 08:56

const INSTRUCTIONS_TXT_CONTENT: &'static str = "Welcome to Fuschia!\n";
const INSTRUCTIONS_TXT_ATTR: FileAttr = FileAttr {
    ino: 3,
    size: INSTRUCTIONS_TXT_CONTENT.len() as u64,
    blocks: 1,
    atime: CREATE_TIME,
    mtime: CREATE_TIME,
    ctime: CREATE_TIME,
    crtime: CREATE_TIME,
    kind: FileType::RegularFile,
    perm: 0o644,
    nlink: 1,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
};

const HELLO_DIR_ATTR: FileAttr = FileAttr {
    ino: 1,
    size: 0,
    blocks: 0,
    atime: CREATE_TIME,
    mtime: CREATE_TIME,
    ctime: CREATE_TIME,
    crtime: CREATE_TIME,
    kind: FileType::Directory,
    perm: 0o755,
    nlink: 2,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
};

const HELLO_TXT_CONTENT: &'static str = "Hello World!\n";

const HELLO_TXT_ATTR: FileAttr = FileAttr {
    ino: 2,
    size: 13,
    blocks: 1,
    atime: CREATE_TIME,
    mtime: CREATE_TIME,
    ctime: CREATE_TIME,
    crtime: CREATE_TIME,
    kind: FileType::RegularFile,
    perm: 0o644,
    nlink: 1,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
};

struct HelloFS;

impl Filesystem for HelloFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if parent == 1 && name.to_str() == Some("hello.txt") {
            reply.entry(&TTL, &HELLO_TXT_ATTR, 0);
        } else if parent == 1 && name.to_str() == Some("instructions.txt") {
            reply.entry(&TTL, &INSTRUCTIONS_TXT_ATTR, 0);
        } else {
            reply.error(ENOENT);
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match ino {
            1 => reply.attr(&TTL, &HELLO_DIR_ATTR),
            2 => reply.attr(&TTL, &HELLO_TXT_ATTR),
            3 => reply.attr(&TTL, &INSTRUCTIONS_TXT_ATTR),
            _ => reply.error(ENOENT),
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        _size: u32,
        reply: ReplyData,
    ) {
        if ino == 2 {
            reply.data(&HELLO_TXT_CONTENT.as_bytes()[offset as usize..]);
        } else if ino == 3 {
            reply.data(&INSTRUCTIONS_TXT_CONTENT.as_bytes()[offset as usize..]);
        } else {
            reply.error(ENOENT);
        }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        if ino != 1 {
            reply.error(ENOENT);
            return;
        }

        let entries = vec![
            (1, FileType::Directory, "."),
            (1, FileType::Directory, ".."),
            (2, FileType::RegularFile, "hello.txt"),
            (3, FileType::RegularFile, "instructions.txt"),
        ];

        // Offset of 0 means no offset.
        // Non-zero offset means the passed offset has already been seen, and we should start after
        // it.
        let to_skip = if offset == 0 { offset } else { offset + 1 } as usize;
        for (i, entry) in entries.into_iter().enumerate().skip(to_skip) {
            reply.add(entry.0, i as i64, entry.1, entry.2);
        }
        reply.ok();
    }
}

fn main() {
    let game_dir: GameDir = dir("cool_dir").with_file(file("cool_file.txt").content("content"));
    env_logger::init();
    let mountpoint = env::args_os().nth(1).unwrap();
    let options = ["-o", "ro", "-o", "fsname=hello"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();
    fuse::mount(HelloFS, &mountpoint, &options).unwrap();
}
