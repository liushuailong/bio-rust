extern crate core;

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;


// vcf 格式的解读参考：
// https://blog.csdn.net/genome_denovo/article/details/78697679

pub enum MetaInfo {

}
#[derive(Debug)]
pub struct Header {
    pub file_format: String,
    pub info: Vec<String>,
    pub filter: Vec<String>,
    pub format: Vec<String>,
    pub other: Vec<String>,


}

impl Header {
    pub fn new() -> Self {
        Self {
            file_format: "".to_string(),
            info: Vec::new(),
            filter: Vec::new(),
            format: Vec::new(),
            other: Vec::new(),
        }
    }
    // todo： 解析header
    pub fn push(&mut self, item: &str) {
        // 判断是否是以#file_Format开头
        if item.starts_with("##fileformat=") {
            self.file_format = item.split("=")
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .get(1).unwrap()
                .to_owned();
            return;
        }
        // 如果不是则判断file_format字段是否为空，如果为空则panic
        if self.file_format.is_empty() {
            panic!("The version of vcf is not exist!!!")
        }
        // info
        if item.starts_with("##INFO=") {
            self.info.push(item.to_string());
            return;
        }
        // filter
        if item.starts_with("##FILTER=") {
            self.filter.push(item.to_string());
            return;
        }
        // format
        if item.starts_with("##FORMAT=") {
            self.format.push(item.to_string());
            return;
        }

        // other
        self.other.push(item.to_string());
    }
    fn display(&self) -> Vec<String> {
        let mut ret = Vec::new();
        ret.push(format!("##fileformat={}", self.file_format).to_string());
        for item in self.info.iter() {
            ret.push(item.to_string());
        }
        for item in self.filter.iter() {
            ret.push(item.to_string());
        }
        for item in self.format.iter() {
            ret.push(item.to_string());
        }
        for item in self.other.iter() {
            ret.push(item.to_string());
        }
        ret
    }


}

// todo： 突变每个字段的数据格式核对
#[derive(Debug, Clone)]
pub struct Variant {
    pub chrom: String,
    pub pos: usize,
    pub id: String,
    pub r#ref: String, // 注意如何解决名字和关键字冲突
    pub alt: String,
    pub qual: String,
    pub filter: String,
    pub info: BTreeMap<String, String>,
    pub format: Vec<String>,
    pub sample_list: BTreeMap<String,BTreeMap<String,String>>
}

// 在你实现Ord Trait 之前， 你首先必须实现PartialOrd, Eq, PartialEq Trait。
impl PartialEq for Variant {
    fn eq(&self, other: &Self) -> bool {
        self.chrom == other.chrom && self.pos == other.pos && self.r#ref == other.r#ref && self.alt == other.alt
    }
}

impl Eq for Variant {}

impl PartialOrd for Variant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let ord_chr = vec!["chrM", "chr1", "chr2", "chr3", "chr4", "chr5", "chr6", "chr7", "chr8", "chr9", "chr10", "chr11", "chr12", "chr13", "chr14", "chr15", "chr16", "chr17", "chr18", "chr19", "chr20", "chr21", "chr22", "chrX", "chrY"];
        let self_pos = ord_chr.iter().position(|&r| r == self.chrom).unwrap();
        let other_pos = ord_chr.iter().position(|&r| r == other.chrom).unwrap();
        if self_pos == other_pos {
            if self.pos == other.pos {
                match self.r#ref.cmp(&other.r#ref) {
                    Ordering::Equal => {
                        match self.alt.cmp(&other.alt) {
                            Ordering::Equal => {
                                Some(Ordering::Equal)
                            }
                            Ordering::Less => {
                                Some(Ordering::Less)
                            }
                            Ordering::Greater => {
                                Some(Ordering::Greater)
                            }
                        }
                    }
                    Ordering::Less => {
                        Some(Ordering::Less)
                    }
                    Ordering::Greater => {
                        Some(Ordering::Greater)
                    }
                }
            } else if self.pos > other.pos {
                Some(Ordering::Greater)
            } else {
                Some(Ordering::Less)
            }
        }
        else if self_pos > other_pos {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Less)
        }
    }
}

impl Ord for Variant {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}


impl Variant {
    fn from(line: &str, title: Vec<String>) -> Result<Self, Box<dyn Error>> {
        let content: Vec<String> = line.clone().split("\t").map(|item| item.to_string()).collect();
        let chrom = content.get(0).unwrap().to_string();
        let pos = content.get(1).unwrap().parse::<usize>().unwrap();
        let id = content.get(2).unwrap().to_string();
        let r#ref = content.get(3).unwrap().to_string();
        let alt = content.get(4).unwrap().to_string();
        let qual = content.get(5).unwrap().to_string();
        let filter = content.get(6).unwrap().to_string();
        let mut info = BTreeMap::new();
        let info_str = content.get(7).unwrap().to_string();
        let into_vec: Vec<String> = info_str.split(";").map(|item| item.to_string()).collect();
        for info_con in into_vec.iter() {
            let mut key_val: Vec<String> = info_con.to_string().split("=").map(|item| item.to_string()).collect();
            if key_val.len() == 1 {
                key_val.push("null".to_string());
            }
            info.insert(key_val.get(0).unwrap().to_string(), key_val.get(1).unwrap().to_string());
        }
        let format : Vec<String> = content.get(8).unwrap().to_string().split(":").map(|item| item.to_string()).collect();
        let mut sample_list = BTreeMap::new();
        let title_len = title.len();
        for i in 9..title_len {
            let content_vec = content.get(i).unwrap()
                .clone()
                .split(":")
                .map(|x| x.to_string())
                .collect::<Vec<String>>();
            let mut temp_cnt = BTreeMap::new();
            for idx in 0..format.len() {
                temp_cnt.insert(format[idx].to_string(), content_vec[idx].to_string());
            }
            // let temp_cnt = format.clone().into_iter().zip(
            //     content.get(i).unwrap().split(":").map(|x| x.to_string()).collect::<Vec<String>>().into_iter()
            // ).collect::<BTreeMap<String,String>>();
            sample_list.insert(title.get(i).unwrap().to_string(), temp_cnt);
        }

        Ok(
            Self {
                chrom,
                pos,
                id,
                r#ref,
                alt,
                qual,
                filter,
                info,
                format,
                sample_list,
            }
        )
    }
    fn display(&self, title: &Vec<String>) -> String {
        let mut ret_vec = vec![
            self.chrom.clone(),
            self.pos.to_string(),
            self.id.clone(),
            self.r#ref.clone(),
            self.alt.clone(),
            self.qual.clone(),
            self.filter.clone(),
        ];
        // 遍历info
        let mut info_vec = Vec::new();
        for (k, v) in &self.info {
            if *v == "null".to_string() {
                info_vec.push(format!("{}",k));
                continue
            }
            info_vec.push(format!("{}={}",k, v));

        }
        ret_vec.push(info_vec.join(";").to_string());
        ret_vec.push(self.format.join(":").to_string());
        // 遍历 self.sample_list;
        // todo: 这个Btree是按照key来排序的，如果是都个样品可能造成内容不对应
        // for (k, v) in &self.sample_list {
        //     let mut format_vec = Vec::new();
        //     for format_item in self.format.iter() {
        //         format_vec.push(v.get(format_item).unwrap().to_string())
        //     }
        //     ret_vec.push(format_vec.join(":"));
        // }
        for idx in 9..title.len() {
            let mut format_vec = Vec::new();
            let k = title.get(idx).unwrap();
            let v = self.sample_list.get(k).unwrap();
            for format_item in self.format.iter() {
                format_vec.push(v.get(format_item).unwrap().to_string())
            }
            ret_vec.push(format_vec.join(":"));
        }
        ret_vec.join("\t").to_string()
    }
}

pub struct Vcf {
    pub header: Header,
    pub title: Vec<String>,
    pub variants: Vec<Variant>,
}

impl Vcf {
    pub fn from(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut header = Header::new();
        let mut title = Vec::new();
        let mut variants: Vec<Variant> = Vec::new();
        // 读取文件内容
        let brd = BufReader::new(File::open(file_path)?);
        for line in brd.lines() {
            if let Ok(line) = line {
                // 片段是开头还是内容
                match &line[..1] {
                    "#" => {
                        match &line[..2] {
                            "##" => {
                                header.push(&line);
                            }
                            "#C" => {
                                title = line.split("\t").map(|x| x.to_string()).collect();
                            }
                            _ => {}
                        }
                    }
                    _ => {
                        variants.push(Variant::from(&line, title.clone())?);
                    }
                }
            } else {
                break;
            }
        }
        Ok(Self {
            header,
            title,
            variants,
        })
    }

    // todo 对vcf进行排序
    // 1. 染色体排序规则 chrM>chr1>chr2>chr3>chr4>chr5>chr6>chr7>chr8>chr9>chr10>chr11>chr12>chr13>chr14>chr15>chr16>chr17>chr18>chr19>chr20>chr21>chr22>chrX>chrY
    pub fn sort(&mut self) {
        self.remove_chrun();
        self.variants.sort();
    }

    fn remove_chrun(&mut self) {
        let ord_chr = vec!["chrM", "chr1", "chr2", "chr3", "chr4", "chr5", "chr6", "chr7", "chr8", "chr9", "chr10", "chr11", "chr12", "chr13", "chr14", "chr15", "chr16", "chr17", "chr18", "chr19", "chr20", "chr21", "chr22", "chrX", "chrY"];
        self.variants = self.variants.iter()
            .filter(|v| ord_chr.iter().position(|&r| r == v.chrom ).is_some())
            .map(|x| x.clone())
            .collect::<Vec<Variant>>();
    }

    // 对vcf实现遍历!!!
    // iter method
    pub fn iter(&self) -> VcfIter {
        // VcfIter::new(self)
        VcfIter {
            header: &self.header,
            title: &self.title,
            variants: &self.variants,
        }
    }
    pub fn display(&self) -> Vec<String> {
        let mut ret_vec = Vec::new();
        for item in self.header.display() {
            ret_vec.push(item);
        }
        ret_vec.push(self.title.join("\t").to_string());
        for item in self.variants.iter() {
            ret_vec.push(item.display(&self.title));
        }
        ret_vec
    }

    pub fn to_file(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(file_path).expect(&format!("create file fail: {}", file_path));
        for item in self.display() {
            file.write(item.as_ref())?;
            file.write("\n".as_ref())?;
        }
        Ok(())
    }
}

pub struct VcfIter<'a> {
    header: &'a Header,
    title: &'a Vec<String>,
    variants: &'a [Variant],
}


impl Iterator for VcfIter<'_> {
    type Item = Variant;
    fn next(&mut self) -> Option<Self::Item>{
        match self.variants.get(0) {
            None => { None }
            Some(variant) => {
                self.variants = &self.variants[1..];
                Some(variant.clone())
            }
        }
    }
}



