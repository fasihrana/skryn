use std::io::prelude::*;
use std::fs::File;

#[derive(Clone, Debug, PartialEq)]
pub enum ImagePath{
    Local(String),
    URL(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Image{
    path: ImagePath,
    bytes: Vec<u8>,
    ext_id: u64,
}

impl Image {
    pub fn load(path: ImagePath) -> Option<Image> {
        let mut ret = None;
        if let ImagePath::Local(_s) = path.clone() {
            let mut f = File::open(&_s[0..]);
            if let Ok(mut c) = f {
                let l = c.metadata().unwrap().len();
                let mut bytes :Vec<u8> = Vec::new();
                bytes.resize(l as usize, 0);
                let _r = c.read(&mut bytes);
                println!("read file ? {:?}", _r);
                if let Ok(_) = _r {
                    //let bytes = contents.to_vec();
                    ret = Some(Image{
                        path,
                        bytes,
                        ext_id:0,
                    });
                }
            }
        }

        ret
    }
}
