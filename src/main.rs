#![feature(const_str_len)]

extern crate env_logger;
extern crate fuse;
extern crate libc;
extern crate time;

use fuse::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, ReplyWrite,
    Request,
};
use libc::ENOENT;
use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use time::Timespec;

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct GameFile {
    name: String,
    inode: u64,
    content: String,
    life: i32,
}
impl GameFile {
    pub fn new(inode: u64, name: String, content: String) -> GameFile {
        GameFile {
            name: name,
            inode: inode,
            content: content,
            life: 5,
        }
    }
    pub fn content(mut self, content: &str) -> Self {
        self.content = content.to_string();
        self
    }
    pub fn get_content(&self) -> String {
        if self.life > 0 {
            self.content.to_string()
        } else {
            "123 I'm dead :(\n".to_string()
        }
    }
    pub fn dec_life(&mut self) {
        self.life -= 1;
    }
    pub fn to_file_attr(&self) -> FileAttr {
        FileAttr {
            ino: self.inode,
            size: self.get_content().len() as u64,
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

#[derive(Debug, Eq, Hash, PartialEq)]
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
    pub fn to_references<'a>(&'a mut self) -> Vec<(u64, DirOrFile<'a>)> {
        let mut vec = Vec::new();
        for file in self.files.iter_mut() {
            vec.push((file.inode, DirOrFile::File(file)));
        }
        for subdir in self.sub_dirs.iter_mut() {
            vec.extend(subdir.to_references());
        }
        vec
    }
}

pub fn dir(inode: u64, name: &str) -> GameDir {
    GameDir::new(inode, name.to_string())
}
pub fn file(inode: u64, name: &str) -> GameFile {
    GameFile::new(inode, name.to_string(), "".to_string())
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum DirOrFile<'a> {
    Dir(&'a GameDir),
    File(&'a mut GameFile),
}

const TTL: Timespec = Timespec { sec: 1, nsec: 0 }; // 1 second

const CREATE_TIME: Timespec = Timespec {
    sec: 1381237736,
    nsec: 0,
}; // 2013-10-08 08:56

pub struct HelloFS<'a> {
    inode_table: HashMap<u64, DirOrFile<'a>>,
}

impl<'a> Filesystem for HelloFS<'a> {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        match self.inode_table.get(&parent) {
            Some(DirOrFile::Dir(d)) => {
                let result = d
                    .files
                    .iter()
                    .find(|f| Some(f.name.as_ref()) == name.to_str());
                if result.is_some() {
                    reply.entry(&TTL, &result.unwrap().to_file_attr(), 0);
                } else {
                    let result = d
                        .sub_dirs
                        .iter()
                        .find(|dir| Some(dir.name.as_ref()) == name.to_str());
                    if result.is_some() {
                        reply.entry(&TTL, &result.unwrap().to_file_attr(), 0);
                    } else {
                        reply.error(ENOENT);
                    }
                }
            }
            _ => reply.error(ENOENT),
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match self.inode_table.get(&ino) {
            Some(DirOrFile::Dir(d)) => reply.attr(&TTL, &d.to_file_attr()),
            Some(DirOrFile::File(f)) => reply.attr(&TTL, &f.to_file_attr()),
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
        match self.inode_table.get_mut(&ino) {
            Some(DirOrFile::Dir(_d)) => reply.error(ENOENT),
            Some(DirOrFile::File(f)) => {
                //let result: () = f;
                //result.dec_life();
                //result.dec_life();
                //reply.data(&f.content.as_bytes()[offset as usize..])
                reply.error(ENOENT)
            }
            None => reply.error(ENOENT),
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
        println!("readdir: ino={} offset={}", ino, offset);
        match self.inode_table.get(&ino) {
            Some(DirOrFile::Dir(gamedir)) => {
                let mut entries: Vec<(u64, FileType, &str)> = Vec::new();
                entries.push((11, FileType::Directory, "."));
                entries.push((12, FileType::Directory, ".."));
                for subdir in gamedir.sub_dirs.iter() {
                    entries.push((subdir.inode, FileType::Directory, subdir.name.as_str()));
                }
                for file in gamedir.files.iter() {
                    entries.push((file.inode, FileType::RegularFile, file.name.as_str()));
                }
                // Offset of 0 means no offset.
                // Non-zero offset means the passed offset has already been seen, and we should start after
                // it.
                let to_skip = if offset == 0 { offset } else { offset + 1 } as usize;
                for (i, entry) in entries.into_iter().enumerate().skip(to_skip) {
                    println!("Adding entry: {} {}", entry.0, entry.2);
                    reply.add(entry.0, i as i64, entry.1, entry.2);
                }
                reply.ok();
            }
            _ => {
                reply.error(ENOENT);
                return;
            }
        }
    }

    fn write(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _offset: i64,
        _data: &[u8],
        _flags: u32,
        reply: ReplyWrite,
    ) {
        reply.error(ENOENT);
    }
}

pub fn gamedir_to_hash_map(gamedir: &mut GameDir) -> HashMap<u64, DirOrFile> {
    let mut hash_map = HashMap::new();
    for (inode, dir_or_file) in gamedir.to_references() {
        hash_map.insert(inode.clone(), dir_or_file);
    }
    hash_map
}

fn main() {
    let mut game_dir: GameDir = dir(1, "cool_dir")
        .with_file(file(3, "cool_file.txt").content("content\n"))
        .with_dir(dir(4, "deep_dir").with_file(file(5, "deep_file.txt").content("deep\n")));
    env_logger::init();
    let mountpoint = env::args_os().nth(1).unwrap();
    let options = ["-o", "ro", "-o", "fsname=hello"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();
    let inode_table = gamedir_to_hash_map(&mut game_dir);
    fuse::mount(
        HelloFS {
            inode_table: inode_table,
        },
        &mountpoint,
        &options,
    )
    .unwrap();
}
