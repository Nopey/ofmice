use ofmice::*;
use download::Index;
use std::env::args;
use std::fs::{File, rename, OpenOptions};
use std::io::Write;

fn main(){
    let mut a = args().skip(1);
    let www = a.next().expect("Expected first of 2 arguments, the www ofmice dir");
    let patch_tail: u32 = a.next().expect("Expected second of 2 arguments, the patch tail len").parse().unwrap();
    let real_name = format!("{}/index.json", www);
    let temp_name = format!("{}/staging/index.json", www);
    let file = File::open(&real_name).unwrap();
    let mut index: Index = serde_json::from_reader(file).unwrap();

    let mut del = OpenOptions::new().append(true).create(true)
        .open("pending-deletion.txt").unwrap();

    // find all the tarballs
    for (bin, bindex) in index.bindices.iter_mut() {
        if bindex.patch_tail < patch_tail {
            bindex.patch_tail += 1;
        } else {
            writeln!(del, "{}{}-patch{}.tar.xz", www, bin, bindex.version - bindex.patch_tail).unwrap();
        }
        
        let from_name = format!("{}.tar.xz.dif", bin);
        // don't need to stage this as we're not overwriting anything
        let to_name = format!("{}{}-patch{}.tar.xz", www, bin, bindex.version);
        rename(&from_name, &to_name).unwrap();
        bindex.version += 1;
    }
    // save index
    let temp = File::create(&temp_name).unwrap();
    serde_json::to_writer_pretty(temp, &index).unwrap();
    // serde_json::to_writer(temp, &index).unwrap();
}
