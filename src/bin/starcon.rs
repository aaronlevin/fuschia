#![feature(const_str_len)]

extern crate env_logger;
extern crate fuse;
extern crate libc;
extern crate time;

use fuse::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, ReplyOpen,
    Request,
};
use libc::ENOENT;
use std::env;
use std::ffi::OsStr;
use time::Timespec;

const TTL: Timespec = Timespec { sec: 2, nsec: 0 }; // 1 second

const CREATE_TIME: Timespec = Timespec {
    sec: 1381237736,
    nsec: 0,
}; // 2013-10-08 08:56

pub fn starcon_content(count: u64) -> String {
    match count % 4 {
        1 => "Hello StarCon!\n".to_string(),
        2 => "Also, good morning!\n".to_string(),
        3 => "Sorry!\n".to_string(),
        4 => ":-)\n".to_string(),
        0 => "FUSE rocks!\n".to_string(),
        _ => "\n".to_string(),
    }
}

pub fn starcon_file_attr(_count: u64) -> FileAttr {
    FileAttr {
        ino: 3,
        //size: starcon_content(count).len() as u64,
        size: 100 as u64,
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

const ROOT_FILE_ATTR: FileAttr = FileAttr {
    ino: 1,
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
};

pub struct StarConFS {
    count: u64,
}

impl Filesystem for StarConFS {
    fn lookup(&mut self, _req: &Request, _parent: u64, _name: &OsStr, reply: ReplyEntry) {
        reply.entry(&TTL, &starcon_file_attr(self.count), 0)
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match ino {
            1 => reply.attr(&TTL, &ROOT_FILE_ATTR),
            3 => reply.attr(&TTL, &starcon_file_attr(self.count)),
            _ => reply.error(ENOENT),
        }
    }

    fn open(&mut self, _req: &Request, _ino: u64, _flags: u32, reply: ReplyOpen) {
        self.count += 1;
        reply.opened(_ino, 0)
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
        if ino == 3 {
            reply.data(&starcon_content(self.count).as_bytes()[offset as usize..])
        } else {
            reply.error(ENOENT)
        }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        let mut entries: Vec<(u64, FileType, &str)> = Vec::new();
        entries.push((11, FileType::Directory, "."));
        entries.push((12, FileType::Directory, ".."));
        entries.push((3, FileType::RegularFile, "starcon.txt"));
        let to_skip = if offset == 0 { offset } else { offset + 1 } as usize;
        for (i, entry) in entries.into_iter().enumerate().skip(to_skip) {
            reply.add(entry.0, i as i64, entry.1, entry.2);
        }
        reply.ok();
    }
}

fn main() {
    let mountpoint = env::args_os().nth(1).unwrap();
    let options = ["-o", "rw", "-o", "fsname=hello"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();
    fuse::mount(StarConFS { count: 0 }, &mountpoint, &options).unwrap();
}
