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

#[derive(Debug)]
pub struct GameFile {
    name: String,
    inode: u64,
    content: String,
    life: u32,
}
impl GameFile {
    pub fn new(inode: u64, name: String, content: String) -> GameFile {
        GameFile {
            name: name,
            inode: inode,
            content: content,
            life: 2,
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
            "I'm dead :(".to_string()
        }
    }
    pub fn dec_life(&mut self) {
        self.life -= 1;
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

#[derive(Debug)]
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

#[derive(Debug)]
pub enum DirOrFile<'a> {
    Dir(&'a GameDir),
    File(&'a GameFile),
}

const TTL: Timespec = Timespec { sec: 1, nsec: 0 }; // 1 second

const CREATE_TIME: Timespec = Timespec {
    sec: 1381237736,
    nsec: 0,
}; // 2013-10-08 08:56

pub struct HelloFS {
    root: GameDir,
}

pub fn lookup_gamedir<'a>(
    parent: u64,
    name: &OsStr,
    gamedir: &'a GameDir,
) -> Option<DirOrFile<'a>> {
    println!("lookup_gamedir({}, {:?})", parent, name);
    if gamedir.inode == parent {
        let result = gamedir
            .files
            .iter()
            .find(|f| Some(f.name.as_ref()) == name.to_str())
            .map(|f| DirOrFile::File(f));
        if result.is_some() {
            result
        } else {
            gamedir
                .sub_dirs
                .iter()
                .find(|dir| Some(dir.name.as_ref()) == name.to_str())
                .map(|d| DirOrFile::Dir(d))
        }
    } else {
        let mut return_val: Option<DirOrFile<'a>> = None;
        for subdir in gamedir.sub_dirs.iter() {
            if return_val.is_none() {
                let result = lookup_gamedir(parent, name, subdir);
                if result.is_some() {
                    return_val = result;
                }
            }
        }
        println!("lookup_gamedir WILL returned: {:?}", return_val);
        return_val
    }
}

pub fn getattr_gamedir(ino: u64, gamedir: &GameDir) -> Option<FileAttr> {
    println!("getattr_gamedir({}, {:?}", ino, gamedir);
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

pub fn read_gamedir(ino: u64, gamedir: &GameDir) -> Option<String> {
    println!("read_gamedir");
    let mut return_val: Option<String> = None;
    for file in gamedir.files.iter() {
        if return_val.is_none() {
            //file.dec_life();
            if file.inode == ino {
                return_val = Some(file.get_content());
            }
        }
    }
    if return_val.is_some() {
        return_val
    } else {
        for subdir in gamedir.sub_dirs.iter() {
            if return_val.is_none() {
                return_val = read_gamedir(ino, &subdir);
            }
        }
        return_val
    }
}

pub fn find_gamedir(ino: u64, root: &GameDir) -> Option<&GameDir> {
    println!("find_gamedir({})", ino);
    let mut return_val: Option<&GameDir> = None;
    if root.inode == ino {
        return_val = Some(root);
    } else {
        for subdir in root.sub_dirs.iter() {
            if return_val.is_none() {
                return_val = find_gamedir(ino, subdir);
            }
        }
    }
    println!("find_gamedir WILL return: {:?}", return_val);
    return_val
}

impl Filesystem for HelloFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        match lookup_gamedir(parent, name, &self.root) {
            Some(DirOrFile::File(f)) => reply.entry(&TTL, &f.to_file_attr(), 0),
            Some(DirOrFile::Dir(d)) => reply.entry(&TTL, &d.to_file_attr(), 0),
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
        match read_gamedir(ino, &self.root) {
            Some(content) => reply.data(&content.as_bytes()[offset as usize..]),
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
        match find_gamedir(ino, &self.root) {
            Some(gamedir) => {
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
            None => {
                reply.error(ENOENT);
                return;
            }
        }
    }
}

fn main() {
    let game_dir: GameDir = dir(1, "cool_dir")
        .with_file(file(3, "cool_file.txt").content("content"))
        .with_dir(dir(4, "deep_dir").with_file(file(5, "deep_file.txt").content("deep")));
    env_logger::init();
    let mountpoint = env::args_os().nth(1).unwrap();
    let options = ["-o", "ro", "-o", "fsname=hello"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();
    fuse::mount(HelloFS { root: game_dir }, &mountpoint, &options).unwrap();
}
