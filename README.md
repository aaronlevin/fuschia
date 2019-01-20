Fuschia
=======

Several [FUSE](https://github.com/libfuse/libfuse) experiments built for [StarCon 2019](https://starcon.io/):

* `starcon`: a FUSE file-system with one file that cycles through content on each read.
* `xml`: a FUSE file-system which will map a (simple) XML file into directories and files.
* `fuschia`: a FUSE file-system *game* where you have to pet kittens by writing `pets` to the `.kitty` files (you can check your status in `LiveJournal.txt`). When you've petted all the kittens, the game is over.

This code was rushed and is unidiomatic rust. It's buggy as hell! No guarantees!
