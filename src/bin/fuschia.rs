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
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::rc::Rc;
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

WOW! YOU GAVE ME ENOUGH PETS!! ❤❤❤❤❤❤❤

     _ _..._ __
    \)`    (` /
     /      `\
    |  d  b   |
    =\  Y    =/--..-="````"-.
      '.=__.-'               `\
         o/                 /\ \
          |                 | \ \   / )
           \    .--""`\    <   \ '-' /
          //   |      ||    \   '---'
         ((,,_/      ((,,___/

"#,
        name
    )
}

pub fn no_more_pets(name: &String) -> String {
    format!(
        r#"Hello StarCon!
My name is: {}

MY HEART IS FICKLE! NO MORE PETS!!!!

      ,-~-,       ,-~~~~-,    /\  /\
(\   / ,-, \    ,'        ', /  ~~  \
 \'-' /   \ \  /     _      #  <0 0> \
  '--'     \ \/    .' '.    # =  Y  =/
            \     / \   \   `#-..!.-'
             \   \   \   `\ \\
              )  />  /     \ \\
             / /`/ /`__     \ \\__
            (____)))_)))     \__)))

"#,
        name
    )
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
            need_pets_content(&self.name, self.life)
        } else if self.life == 0 {
            happy_kitty_content(&self.name)
        } else {
            no_more_pets(&self.name)
        }
    }
}

pub fn file(inode: u64, name: &str) -> GameFile {
    GameFile::new(inode, name.to_string(), "".to_string())
}

pub struct GameStatus {
    kitties_needing_pets: u32,
    kitties_at_peace: u32,
    kitties_mad: u32,
}
impl GameStatus {
    pub fn to_content(&self) -> String {
        if self.kitties_needing_pets == 0 && self.kitties_mad == 0 {
            format!(
                r#"GAME OVER!!

All the kitties are at peace!!!

             *     ,MMM8&&&.            *
                  MMMM88&&&&&    .
                 MMMM88&&&&&&&
     *           MMM88&&&&&&&&
                 MMM88&&&&&&&&
                 'MMM88&&&&&&'
                   'MMM8&&&'      *
          |\___/|
          )     (             .              '
         =\     /=
           )===(       *
          /     \
          |     |
         /       \
         \       /
  _/\_/\_/\__  _/_/\_/\_/\_/\_/\_/\_/\_/\_/\_
  |  |  |  |( (  |  |  |  |  |  |  |  |  |  |
  |  |  |  | ) ) |  |  |  |  |  |  |  |  |  |
  |  |  |  |(_(  |  |  |  |  |  |  |  |  |  |
  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
  "#
            )
        } else if self.kitties_needing_pets == 0 {
            format!(
                r#"GAME OVER!!!

SO MANY KITIES ARE MAD AT U!!!!!!!!!!!! :-(
       ___
   _.-|   |          |\__/,|   (`\
  (   | {} |          |o o  |__ _) )
   "-.|___|        _.( T   )  `  /
    .--'-`-.     _((_ `^--' /_<  \
  .+|______|__.-||__)`-'(((/  (((/

        "#,
                self.kitties_mad
            )
        } else {
            format!(
                r#"Dear Diary,

All my friends are at StarCon! :(

I have to stay at home and pet these kitties :~(

Here's what I've done so far:

* {} kitties still need pets
* {} kitties are at peace with the world
* {} kitties are mad because I petted them too much!
"#,
                self.kitties_needing_pets, self.kitties_at_peace, self.kitties_mad
            )
        }
    }
}

const TTL: Timespec = Timespec { sec: 1, nsec: 0 }; // 1 second

const CREATE_TIME: Timespec = Timespec {
    sec: 1381237736,
    nsec: 0,
}; // 2013-10-08 08:56

pub struct FuschiaFS {
    //gamedir: Rc<RefCell<GameDir>>,
    inode_table: HashMap<u64, Either>,
    parent_table: HashMap<u64, Vec<Either>>,
}
impl FuschiaFS {
    pub fn game_status(&self) -> GameStatus {
        let mut needing_pets_count: u32 = 0;
        let mut at_peace_count: u32 = 0;
        let mut mad_count: u32 = 0;

        for (_, either) in self.inode_table.iter() {
            match either {
                Either::Directory { .. } => {}
                Either::File { file: file_ref } => {
                    let borrowed_file = (*file_ref).borrow();
                    let name = &borrowed_file.name;
                    let pets_needed = borrowed_file.life;
                    if name != "LiveJournal.txt" {
                        if pets_needed > 0 {
                            needing_pets_count += 1;
                        } else if pets_needed == 0 {
                            at_peace_count += 1;
                        } else {
                            mad_count += 1;
                        }
                    }
                }
            }
        }
        GameStatus {
            kitties_needing_pets: needing_pets_count,
            kitties_at_peace: at_peace_count,
            kitties_mad: mad_count,
        }
    }

    pub fn to_file_attr(&self, either: &Either) -> FileAttr {
        match either {
            Either::Directory { dir: d } => {
                let borrowed_dir = d.borrow();
                FileAttr {
                    ino: borrowed_dir.inode,
                    size: 0,
                    blocks: 1,
                    atime: CREATE_TIME,
                    mtime: CREATE_TIME,
                    ctime: CREATE_TIME,
                    crtime: CREATE_TIME,
                    kind: FileType::Directory,
                    perm: 0o644,
                    nlink: 1,
                    uid: 1000,
                    gid: 100,
                    rdev: 0,
                    flags: 0,
                }
            }
            Either::File { file: f } => {
                let borrowed_file = f.borrow();
                let mut content_size;
                if borrowed_file.name == "LiveJournal.txt" {
                    let game_status = self.game_status();
                    content_size = game_status.to_content().len() as u64;
                } else {
                    content_size = borrowed_file.get_content().len() as u64;
                }
                FileAttr {
                    ino: borrowed_file.inode,
                    size: content_size,
                    blocks: 1,
                    atime: CREATE_TIME,
                    mtime: CREATE_TIME,
                    ctime: CREATE_TIME,
                    crtime: CREATE_TIME,
                    kind: FileType::RegularFile,
                    perm: 0o644,
                    nlink: 1,
                    uid: 1000,
                    gid: 100,
                    rdev: 0,
                    flags: 0,
                }
            }
        }
    }
}

impl Filesystem for FuschiaFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        match self.parent_table.get(&parent) {
            Some(children) => {
                let filtered = children
                    .iter()
                    .filter(|c| c.name().as_str() == name.to_str().unwrap())
                    .collect::<Vec<&Either>>();
                if filtered.len() == 1 {
                    let child = filtered.get(0).unwrap();
                    reply.entry(&TTL, &self.to_file_attr(&child), 0);
                } else {
                    reply.error(ENOENT);
                }
            }
            _ => reply.error(ENOENT),
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match self.inode_table.get(&ino) {
            Some(dir_or_file) => reply.attr(&TTL, &self.to_file_attr(&dir_or_file)),
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
        match self.inode_table.get(&ino) {
            Some(Either::File { file: f }) => {
                let borrowed_file = f.borrow();
                if borrowed_file.name == "LiveJournal.txt" {
                    let status = self.game_status();
                    reply.data(&status.to_content().as_bytes()[offset as usize..])
                } else {
                    reply.data(&borrowed_file.get_content().as_bytes()[offset as usize..])
                }
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
            Some(Either::Directory { dir: d }) => {
                let mut entries: Vec<(u64, FileType, String)> = Vec::new();

                entries.push((1111, FileType::Directory, ".".to_string()));
                entries.push((11112, FileType::Directory, "..".to_string()));

                let borrowed_directory = d.borrow();

                for subdir in borrowed_directory.sub_dirs.iter() {
                    let borrowed = subdir.borrow();
                    entries.push((borrowed.inode, FileType::Directory, borrowed.name.clone()));
                }
                for file in borrowed_directory.files.iter() {
                    let borrowed = file.borrow();
                    entries.push((borrowed.inode, FileType::RegularFile, borrowed.name.clone()));
                }
                let to_skip = if offset == 0 { offset } else { offset + 1 } as usize;
                for (i, entry) in entries.into_iter().enumerate().skip(to_skip) {
                    reply.add(entry.0, i as i64, entry.1, entry.2.as_str());
                }
                reply.ok();
            }
            _ => reply.error(ENOENT),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct GameDir {
    inode: u64,
    name: String,
    files: Vec<Rc<RefCell<GameFile>>>,
    sub_dirs: Vec<Rc<RefCell<GameDir>>>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Either {
    Directory { dir: Rc<RefCell<GameDir>> },
    File { file: Rc<RefCell<GameFile>> },
}
impl Either {
    pub fn name(&self) -> String {
        match self {
            Either::File { file: f } => f.borrow().name.clone(),
            Either::Directory { dir: d } => d.borrow().name.clone(),
        }
    }
}

pub fn update_inode_map(gamedir: &Rc<RefCell<GameDir>>, hash_map: &mut HashMap<u64, Either>) {
    // not sure how safe this is? is borrowed_gamedir
    // borrowed twice?
    let borrowed_gamedir = (*gamedir).borrow();

    hash_map.insert(
        borrowed_gamedir.inode,
        Either::Directory {
            dir: Rc::clone(&gamedir),
        },
    );
    for file in borrowed_gamedir.files.iter() {
        let inode: u64 = (*file).borrow().inode;
        hash_map.insert(
            inode,
            Either::File {
                file: Rc::clone(&file),
            },
        );
    }
    for subdir in borrowed_gamedir.sub_dirs.iter() {
        update_inode_map(subdir, hash_map);
    }
}

pub fn update_parent_map(gamedir: &Rc<RefCell<GameDir>>, hash_map: &mut HashMap<u64, Vec<Either>>) {
    let borrowed_gamedir = (*gamedir).borrow();
    let mut vec =
        Vec::with_capacity(borrowed_gamedir.files.len() + borrowed_gamedir.sub_dirs.len());
    vec.extend(
        borrowed_gamedir
            .files
            .iter()
            .map(|f| Either::File { file: Rc::clone(f) }),
    );
    vec.extend(
        borrowed_gamedir
            .sub_dirs
            .iter()
            .map(|d| Either::Directory { dir: Rc::clone(d) }),
    );

    hash_map.insert(borrowed_gamedir.inode, vec);

    // recurse
    for subdir in borrowed_gamedir.sub_dirs.iter() {
        update_parent_map(subdir, hash_map);
    }
}

fn main() {
    let game_dir = Rc::new(RefCell::new(GameDir {
        inode: 1,
        name: "cool".to_string(),
        files: [
            Rc::new(RefCell::new(file(2, "LiveJournal.txt"))),
            Rc::new(RefCell::new(file(3, "3.txt"))),
        ]
        .to_vec(),
        sub_dirs: [
            Rc::new(RefCell::new(GameDir {
                inode: 4,
                name: "xxx".to_string(),
                files: [
                    Rc::new(RefCell::new(file(5, "5.txt"))),
                    Rc::new(RefCell::new(file(6, "6.txt"))),
                ]
                .to_vec(),
                sub_dirs: [Rc::new(RefCell::new(GameDir {
                    inode: 7,
                    name: "xxxxx".to_string(),
                    files: [Rc::new(RefCell::new(file(8, "8.txt")))].to_vec(),
                    sub_dirs: [].to_vec(),
                }))]
                .to_vec(),
            })),
            Rc::new(RefCell::new(GameDir {
                inode: 9,
                name: "lskdjf".to_string(),
                files: [Rc::new(RefCell::new(file(10, "10.txt")))].to_vec(),
                sub_dirs: [].to_vec(),
            })),
        ]
        .to_vec(),
    }));
    let mut inode_table = HashMap::new();
    let mut parent_table = HashMap::new();
    update_inode_map(&game_dir, &mut inode_table);
    update_parent_map(&game_dir, &mut parent_table);

    env_logger::init();
    let mountpoint = env::args_os().nth(1).unwrap();
    let options = ["-o", "rw", "-o", "fsname=hello"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();
    fuse::mount(
        FuschiaFS {
            inode_table: inode_table,
            parent_table: parent_table,
        },
        &mountpoint,
        &options,
    )
    .unwrap();
}
