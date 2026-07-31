use antikythera_facade::SimpleAgent;
use std::io::{self, Write};

#[tokio::main]
async fn main() {
    let model = std::env::args().nth(1).unwrap_or_else(|| "gpt-oss:120b-cloud".to_string());

    println!("Antikythera Chat Agent");
    println!("Model: {model}");
    println!("Ketik pesan anda (Ctrl+C untuk keluar):");
    println!();

    let mut agent = match SimpleAgent::ollama(&model).await {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Gagal koneksi: {e}");
            eprintln!("Pastikan Ollama berjalan: ollama serve");
            std::process::exit(1);
        }
    };

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        match agent.chat(input).await {
            Ok(response) => println!("\n{response}\n"),
            Err(e) => eprintln!("Error: {e}\n"),
        }
    }
}
