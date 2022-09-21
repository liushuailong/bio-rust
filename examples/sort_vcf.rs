use clap::Parser;
use bio_rust::vcf::Vcf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    input_vcf: String,
    #[clap(short='o', long="output")]
    output_vcf: String,
}

fn main() {
    let args = Args::parse();
    // 从文件中读取vcf
    let mut vcf = Vcf::from(&args.input_vcf).unwrap();
    // 对vcf进行遍历
    // for variant in vcf.iter() {
    //     println!("{:?}", variant);
    // }
    // 对vcf进行排序
    vcf.sort();
    // println!("{:?}", vcf.variants.get(1).unwrap());
    // println!("{:?}", vcf.variants.get(2).unwrap());
    // 将vcf输出到vcf文件
    vcf.to_file(&args.output_vcf).unwrap();
    // todo: 将vcf以table形式输出
    // todo: 处理没有sample的vcf格式
    // todo: 对vcf进行格式化
    println!("Hello, world!");
}
