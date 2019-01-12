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
    inode: u64,
    content: String,
}
impl GameFile {
    pub fn new(inode: u64, name: String, content: String) -> GameFile {
        GameFile {
            name: name,
            inode: inode,
            content: content,
        }
    }
    pub fn content(mut self, content: &str) -> Self {
        self.content = content.to_string();
        self
    }
    pub fn to_file_attr(&self) -> FileAttr {
        FileAttr {
            ino: self.inode,
            size: self.content.len() as u64,
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
        }
    }
}

pub struct GameDir {
    name: String,
    inode: u64,
    files: Vec<GameFile>,
    sub_dirs: Vec<GameDir>,
}
impl GameDir {
    pub fn new(inode: u64, name: String) -> GameDir {
        GameDir {
            name: name,
            inode: inode,
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
    pub fn to_file_attr(&self) -> FileAttr {
        FileAttr {
            ino: self.inode,
            size: 0,
            blocks: 1,
            atime: CREATE_TIME,
            mtime: CREATE_TIME,
            ctime: CREATE_TIME,
            crtime: CREATE_TIME,
            kind: FileType::Directory,
            perm: 0o644,
            nlink: 1,
            uid: 501,
            gid: 20,
            rdev: 0,
            flags: 0,
        }
    }
}

pub fn dir(inode: u64, name: &str) -> GameDir {
    GameDir::new(inode, name.to_string())
}
pub fn file(inode: u64, name: &str) -> GameFile {
    GameFile::new(inode, name.to_string(), "".to_string())
}

const TTL: Timespec = Timespec { sec: 1, nsec: 0 }; // 1 second

const CREATE_TIME: Timespec = Timespec {
    sec: 1381237736,
    nsec: 0,
}; // 2013-10-08 08:56

const INSTRUCTIONS_TXT_CONTENT: &'static str = "Welcome to Fuschia!\n";

const HELLO_TXT_CONTENT: &'static str = "Hello World!\n";

pub struct HelloFS {
    root: GameDir,
}

pub fn lookup_gamedir<'a>(parent: u64, name: &OsStr, gamedir: &'a GameDir) -> Option<&'a GameFile> {
    if gamedir.inode == parent {
        gamedir
            .files
            .iter()
            .find(|f| Some(f.name.as_ref()) == name.to_str())
    } else {
        let mut return_val: Option<&'a GameFile> = None;
        for subdir in gamedir.sub_dirs.iter() {
            if return_val.is_none() {
                let result = lookup_gamedir(parent, name, subdir);
                if result.is_some() {
                    return_val = result;
                }
            }
        }
        return_val
    }
}

pub fn getattr_gamedir(ino: u64, gamedir: &GameDir) -> Option<FileAttr> {
    if gamedir.inode == ino {
        Some(gamedir.to_file_attr())
    } else {
        let mut return_val: Option<FileAttr> = None;
        for file in gamedir.files.iter() {
            if return_val.is_none() && file.inode == ino {
                return_val = Some(file.to_file_attr());
            }
        }
        for subdir in gamedir.sub_dirs.iter() {
            if return_val.is_none() {
                let result = getattr_gamedir(ino, subdir);
                if result.is_some() {
                    return_val = result;
                }
            }
        }
        return_val
    }
}

impl Filesystem for HelloFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        match lookup_gamedir(parent, name, &self.root) {
            Some(f) => reply.entry(&TTL, &f.to_file_attr(), 0),
            None => reply.error(ENOENT),
        };
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match getattr_gamedir(ino, &self.root) {
            Some(file_attr) => reply.attr(&TTL, &file_attr),
            None => reply.error(ENOENT),
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
    let game_dir: GameDir =
        dir(11, "cool_dir").with_file(file(12, "cool_file.txt").content("content"));
    env_logger::init();
    let mountpoint = env::args_os().nth(1).unwrap();
    let options = ["-o", "ro", "-o", "fsname=hello"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();
    fuse::mount(HelloFS { root: game_dir }, &mountpoint, &options).unwrap();
}
