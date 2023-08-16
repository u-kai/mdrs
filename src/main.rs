use mdrs::md::Markdown;

fn main() {
    let filename = std::env::args().nth(1).expect("no filename given");
    let content = std::fs::read_to_string(&filename).expect("could not read file");

    let md = Markdown::parse(&content);
    md.components().for_each(|c| println!("{:#?}", c));
}
