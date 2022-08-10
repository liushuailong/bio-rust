//! # bio-rust
//! 解析生物信息领域的基本数据结构，提供操纵这些数据的接口和构建一些统计模型。
use std::io::{prelude::*, BufReader};
use std::error::Error;
use std::fs::File;
use std::path::Path;
use flate2::read::GzDecoder;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::iter::Iterator;


#[derive(Debug)]
pub struct Reads {
    read_id: String,
    sequence: String,
    read_name: String,
    quality: String,
}

impl Reads {
    pub fn len(&self) -> usize{
        self.sequence.len()
    }

    pub fn lt_qc(&self, score: i32) -> i32{
        let bytes = self.quality.as_bytes();
        let x : Vec<i32> = bytes.iter()
            .map(|&i| (i as i32 ) - 33i32)
            .filter(|&i| i >= score)
            .collect();
        x.len() as i32
    }

    fn display(&self) -> String {
        format!("{}\n{}\n{}\n{}\n", self.read_id, self.sequence, self.read_name, self.quality)
    }
}

impl Clone for Reads {
    fn clone(&self) -> Reads {
        Reads {
            read_id: self.read_id.to_string(),
            sequence: self.sequence.to_string(),
            read_name: self.read_name.to_string(),
            quality: self.quality.to_string(),
        }
    }
}

pub struct Fastq {
    reads: Vec<Reads>,
    length: u64,
}

impl Fastq {
    pub fn new(reads: Vec<Reads>) -> Self{
        let length = reads.len() as u64;
        Fastq {
            reads,
            length,
        }
    }

    fn push(&mut self, reads: Reads) {
        self.reads.push(reads);
        self.length += 1u64;
    }

    pub fn total_base_num(&self) -> u64 {
        self.reads.iter().map(|r| r.len()).fold(0u64, |acc, x| acc + x as u64)
    }
    pub fn qc_num(&self, score: i32) -> u64 {
        self.reads.iter().map(|r| r.lt_qc(score)).fold(0u64, |acc, x| acc + x as u64)
    }

    // from file
    pub fn from_file(file_path: &str) -> Result<Self, Box<dyn Error>> {
        // todo 目前只支持fastq.gz格式的输入，需要兼容fastq文本格式的输入。
        let fastq_gz = File::open(file_path).expect(format!("No such file or directory: {}", file_path).as_str());
        let fastq_content = GzDecoder::new(fastq_gz)?;
        let fastq_reader = BufReader::new(fastq_content);
        let mut line_iter = fastq_reader.lines().map(|l| l.unwrap());
        let mut fastq = Fastq::new(Vec::new());
        loop {
            let read_id: String;
            let sequence: String;
            let read_name: String;
            let quality: String;
            match line_iter.next(){
                None => {break;}
                Some(element) => {
                    read_id = element;
                }
            }
            match line_iter.next(){
                None => {break}
                Some(element) => {
                    sequence = element;
                }
            }
            match line_iter.next(){
                None => {break}
                Some(element) => {
                    read_name = element;
                }
            }
            match line_iter.next(){
                None => {break}
                Some(element) => {
                    quality = element;
                }
            }
            fastq.push(Reads{
                read_id,
                sequence,
                read_name,
                quality
            })
        }
        Ok(fastq)
    }

    pub fn extent(&mut self, other_fastq: &Fastq) {
        for reads in other_fastq.reads.iter() {
            self.push(reads.clone())
        }
    }
    // merge fastq
    pub fn merge_fastq(fastq_vec: Vec<&str>) -> Result<Self, Box<dyn Error>> {
        let mut ret_fastq = Fastq::new(Vec::new());
        // todo
        for fastq_path in fastq_vec.iter() {
            // 3. 从压缩文件中读取文件
            let fastq_tmp = Fastq::from_file(fastq_path)?;
            ret_fastq.extent(&fastq_tmp);
        }
        Ok(ret_fastq)
    }

    pub fn to_file(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        // print_fastq_to_file
        let mut out_encoder = GzEncoder::new(Vec::new(), Compression::default());
        for reads in self.reads.iter() {
            out_encoder.write_all(reads.display().as_bytes())?;
        }
        // todo
        let compressed_bytes = out_encoder.finish()?;
        let mut file = File::create(file_path).expect("create failed");
        file.write_all(&compressed_bytes).expect("write failed");
        Ok(())
    }
}

impl Iterator for Fastq {
    type Item = Reads;

    fn next(&mut self) -> Option<Self::Item> {
        match self.reads.iter().next() {
            None => { None }
            Some(reads) => { Some(reads.clone())}
        }
    }
}


#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::error::Error;
    use crate::Fastq;

    #[test]
    fn test_merge_fastq() -> Result<(), Box<dyn Error>>{
        let fastq_file_vec = vec!["data/s1062207050023_1.fastq.gz", "data/s1062207050023_2.fastq.gz"];
        let m_fastq = Fastq::merge_fastq(fastq_file_vec)?;
        m_fastq.to_file("data/s1062207050023.fastq.gz");
        Ok(())
    }

    #[test]
    fn test_iter_fastq() -> Result<(), Box<dyn Error>>{
        let m_fastq = Fastq::from_file("data/s1062207050023_1.fastq.gz")?;
        for read in m_fastq {
            println!("{}", read.display());
            break;
        }
        Ok(())
    }
}
