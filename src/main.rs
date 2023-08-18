use mdrs::{
    md::{Component, Markdown},
    pptx::{ContentConfig, Font, Pptx},
};
use std::fs::read_to_string;

#[tokio::main]
async fn main() {
    let filename = std::env::args().nth(1).unwrap();
    let content = read_to_string(filename).unwrap();
    let md = Markdown::parse(&content);
    let config = ContentConfig::default()
        .normal(Font {
            size: 24,
            bold: false,
        })
        .h1(Font {
            size: 36,
            bold: true,
        })
        .h2(Font {
            size: 28,
            bold: true,
        })
        .h3(Font {
            size: 24,
            bold: true,
        });
    let pptx = Pptx::from_md_with_config(md, "test.pptx", &config);
    println!("pptx: {:#?}", pptx);
    create_pptx(pptx).await;
}

async fn create_pptx(pptx: Pptx) {
    let response = reqwest::Client::new()
        .post("http://127.0.0.1:5000/create_pptx")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&pptx).unwrap())
        .send()
        .await
        .unwrap();
    if response.status().is_success() {
        println!("success");
    } else {
        println!("failed");
        println!("{:#?}", response.text().await.unwrap());
    }
}
