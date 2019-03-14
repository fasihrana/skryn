use std::fs::File;
use std::io::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum ImagePath {
    Local(String),
    URL(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Image {
    path: ImagePath,
    bytes: Vec<u8>,
    ext_id: u64,
}

impl Image {
    pub fn load(path: ImagePath) -> Option<Image> {
        if let ImagePath::Local(_s) = path.clone() {
            let f = File::open(&_s[0..]);
            if let Ok(mut c) = f {
                let l = c.metadata().unwrap().len();
                let mut bytes: Vec<u8> = Vec::new();
                bytes.resize(l as usize, 0);
                let r = c.read(&mut bytes);
                println!("read file ? {:?}", r);
                if r.is_ok() {
                    //let bytes = contents.to_vec();
                    return Some(Image {
                        path,
                        bytes,
                        ext_id: 0,
                    });
                }
            }
        }

        None
    }
}
