#![feature(const_str_len)]

extern crate env_logger;
extern crate fuse;
extern crate libc;
extern crate time;

use fuse::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEmpty, ReplyEntry,
    ReplyWrite, Request,
};
use libc::ENOENT;
use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use time::Timespec;

pub fn need_pets_content(name: &String, pets_needed: i32) -> String {
    format!(
        r#"Hello StarCon!
My name is: {}

I NEED TO BE PETTED

Please send me {} pets
                           __ _..._ _
                           \ `)    `(/
                           /`       \
                           |   d  b  |
             .-"````"=-..--\=    Y  /=
           /`               `-.__=.'
    _     / /\                 /o
   ( \   / / |                 |
    \ '-' /   >    /`""--.    /
     '---'   /    ||      |   \\
             \___,,))      \_,,))
"#,
        name, pets_needed
    )
}

pub fn happy_kitty_content(name: &String) -> String {
    format!(
        r#"Hello StarCon!
My name is: {}

WOW! YOU GAVE ME ENOUGH PETS!! :heart:

                           __ _..._ _
                           \ `)    `(/
                           /`       \
                           |   d  b  |
             .-"````"=-..--\=    Y  /=
           /`               `-.__=.'
    _     / /\                 /o
   ( \   / / |                 |
    \ '-' /   >    /`""--.    /
     '---'   /    ||      |   \\
             \___,,))      \_,,))
"#,
        name
    )
}

pub fn no_more_pets(name: &String) -> String {
    format!(
        r#"Hello StarCon!
My name is: {}

MY HEART IS FICKLE! NO MORE PETS!!!!

                           __ _..._ _
                           \ `)    `(/
                           /`       \
                           |   d  b  |
             .-"````"=-..--\=    Y  /=
           /`               `-.__=.'
    _     / /\                 /o
   ( \   / / |                 |
    \ '-' /   >    /`""--.    /
     '---'   /    ||      |   \\
             \___,,))      \_,,))
"#,
        name
    )
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum GameEntity {
    Directory {
        inode: u64,
        name: String,
        parent: Option<u64>,
        children: Vec<u64>,
    },
    File {
        inode: u64,
        name: String,
        parent: Option<u64>,
        content: String,
        life: i32,
        pets_needed: i32,
    },
}
impl GameEntity {
    pub fn dir(inode: u64, name: &str) -> GameEntity {
        GameEntity::Directory {
            inode: inode,
            name: name.to_string(),
            parent: None,
            children: Vec::new(),
        }
    }
    pub fn file(inode: u64, name: &str, content: &str) -> GameEntity {
        GameEntity::File {
            inode: inode,
            name: name.to_string(),
            parent: None,
            content: content.to_string(),
            life: 100,
            pets_needed: 5,
        }
    }
    pub fn get_name(&self) -> &str {
        match self {
            GameEntity::Directory { inode: _, name, .. } => name.as_str(),
            GameEntity::File { inode: _, name, .. } => name.as_str(),
        }
    }
    pub fn get_inode(&self) -> u64 {
        match self {
            GameEntity::Directory { inode, .. } => *inode,
            GameEntity::File { inode, .. } => *inode,
        }
    }
    pub fn get_content(&self) -> String {
        match self {
            GameEntity::File {
                inode: _,
                name: _,
                parent: _,
                content,
                life: _,
                pets_needed,
            } => {
                if *pets_needed > 0 {
                    need_pets_content(content, *pets_needed)
                } else if *pets_needed == 0 {
                    happy_kitty_content(content)
                } else {
                    no_more_pets(content)
                }
            }
            GameEntity::Directory { .. } => "".to_string(),
        }
    }
    pub fn set_parent(&mut self, parent_inode: u64) {
        match self {
            GameEntity::Directory {
                inode: _,
                name: _,
                ref mut parent,
                ..
            } => {
                *parent = Some(parent_inode);
            }
            GameEntity::File {
                inode: _,
                name: _,
                ref mut parent,
                ..
            } => *parent = Some(parent_inode),
        }
    }
    pub fn push_child(&mut self, child_inode: u64) {
        match self {
            GameEntity::Directory {
                inode: _,
                name: _,
                parent: _,
                children,
            } => children.push(child_inode),
            _ => {}
        }
    }
    pub fn to_file_attr(&self) -> FileAttr {
        match self {
            GameEntity::Directory {
                inode,
                name: _,
                parent: _,
                children: _,
            } => FileAttr {
                ino: *inode,
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
            },
            GameEntity::File {
                inode,
                name: _,
                parent: _,
                content: _,
                life: _,
                pets_needed: _,
            } => FileAttr {
                ino: *inode,
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
            },
        }
    }
}
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

    pub fn to_game_entity(&self) -> GameEntity {
        GameEntity::File {
            inode: self.inode,
            name: self.name.clone(),
            parent: None,
            content: self.content.clone(),
            life: self.life,
            pets_needed: self.life,
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
    pub fn to_game_entities(&self, parent: Option<u64>) -> Vec<GameEntity> {
        let mut vec = Vec::new();
        let mut children_vec = Vec::new();
        for file in self.files.iter() {
            children_vec.push(file.inode);
        }
        for subdir in self.sub_dirs.iter() {
            children_vec.push(subdir.inode);
        }
        let root = GameEntity::Directory {
            inode: self.inode,
            name: self.name.clone(),
            parent: parent,
            children: children_vec,
        };
        vec.push(root);
        for file in self.files.iter() {
            let mut entity = file.to_game_entity();
            println!("entity.set_parent: {:?} {}", entity, self.inode);
            entity.set_parent(self.inode);
            vec.push(entity);
        }
        for subdir in self.sub_dirs.iter() {
            let subdir_vec = subdir.to_game_entities(Some(self.inode));
            vec.extend(subdir_vec);
        }
        vec
    }
    pub fn to_entity_hash_map(&self) -> HashMap<u64, GameEntity> {
        let mut hash_map = HashMap::new();
        for entity in self.to_game_entities(None) {
            hash_map.insert(entity.get_inode(), entity);
        }
        hash_map
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

pub struct HelloFS {
    inode_table: HashMap<u64, GameEntity>,
}

impl Filesystem for HelloFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        match self.inode_table.get(&parent) {
            Some(GameEntity::Directory {
                inode: _,
                name: _,
                parent: _,
                children,
            }) => {
                match children
                    .iter()
                    .map(|child_inode| self.inode_table.get(child_inode))
                    .filter(|some_entity| {
                        some_entity.is_some() && some_entity.map(|e| e.get_name()) == name.to_str()
                    })
                    .collect::<Vec<Option<&GameEntity>>>()
                    .as_slice()
                {
                    [Some(file_or_dir)] => reply.entry(&TTL, &file_or_dir.to_file_attr(), 0),
                    _ => reply.error(ENOENT),
                }
            }
            _ => reply.error(ENOENT),
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match self.inode_table.get(&ino) {
            Some(dir_or_file) => reply.attr(&TTL, &dir_or_file.to_file_attr()),
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
            Some(f @ GameEntity::File { .. }) => {
                reply.data(&f.get_content().as_bytes()[offset as usize..])
            }
            _ => reply.error(ENOENT),
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
        match self.inode_table.get(&ino) {
            Some(GameEntity::Directory {
                inode: _,
                name: _,
                parent: _,
                children,
            }) => {
                let mut entries: Vec<(u64, FileType, &str)> = Vec::new();
                entries.push((11, FileType::Directory, "."));
                entries.push((12, FileType::Directory, ".."));
                for child in children.iter() {
                    match self.inode_table.get(child) {
                        Some(GameEntity::Directory { inode, name, .. }) => {
                            entries.push((*inode, FileType::Directory, name.as_str()))
                        }
                        Some(GameEntity::File { inode, name, .. }) => {
                            entries.push((*inode, FileType::RegularFile, name.as_str()))
                        }
                        None => {}
                    }
                }
                let to_skip = if offset == 0 { offset } else { offset + 1 } as usize;
                for (i, entry) in entries.into_iter().enumerate().skip(to_skip) {
                    reply.add(entry.0, i as i64, entry.1, entry.2);
                }
                reply.ok();
            }
            _ => reply.error(ENOENT),
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
        match self.inode_table.get_mut(&_ino) {
            Some(GameEntity::File {
                inode: _,
                name: _,
                parent: _,
                content: _,
                life: _,
                ref mut pets_needed,
            }) => {
                let string = unsafe { std::str::from_utf8_unchecked(_data) };
                if string == "pets\n" || string == "pets" {
                    if *pets_needed < 0 {
                        reply.error(ENOENT);
                    } else {
                        *pets_needed -= 1;
                        reply.written(string.len() as u32);
                    }
                } else {
                    reply.error(ENOENT);
                }
            }
            _ => {
                println!("UH OH ERROR :(. inode: {} handle: {}", _ino, _fh);
                reply.error(ENOENT)
            }
        }
    }

    fn flush(&mut self, _req: &Request, _ino: u64, _fh: u64, _lock_owner: u64, reply: ReplyEmpty) {
        reply.ok();
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
    let game_dir: GameDir = dir(1, "cool_dir")
        .with_file(file(3, "cool_file.txt").content("LAPTOP"))
        .with_dir(dir(4, "deep_dir").with_file(file(5, "deep_file.txt").content("WIFI")));
    let game_entities = game_dir.to_game_entities(None);
    for entity in game_entities.iter() {
        println!("entity: {:?}", entity);
    }
    env_logger::init();
    let mountpoint = env::args_os().nth(1).unwrap();
    let options = ["-o", "rw", "-o", "fsname=hello"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();
    let inode_table = game_dir.to_entity_hash_map();
    fuse::mount(
        HelloFS {
            inode_table: inode_table,
        },
        &mountpoint,
        &options,
    )
    .unwrap();
}
