use clap::{Arg, Command};
use reqwest;
use serde_json::Value;
use std::error::Error;
use std::io::{self};
use regex::Regex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("GitHub File Downloader")
        .version("0.1.0")
        .author("Your Name")
        .about("Downloads files from a GitHub repository URL")
        .arg(
            Arg::new("url")
                .short('u')
                .long("url")
                .value_name("URL")
                .help("Enter the complete GitHub URL of the file"),
        )
        .arg(
            Arg::new("token")
                .short('t')
                .long("token")
                .value_name("TOKEN")
                .help("GitHub personal access token"),
        )
        .get_matches();

    let url = match matches.get_one::<String>("url") {
        Some(u) => u.to_string(),
        None => {
            println!("Enter the complete GitHub URL of the file:");
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        },
    };

    let token = match matches.get_one::<String>("token") {
        Some(t) => t.to_string(),
        None => {
            println!("Enter your GitHub personal access token:");
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        },
    };

    download_from_github(&url, &token).await?;

    Ok(())
}

async fn download_from_github(url: &str, token: &str) -> Result<(), Box<dyn Error>> {
    // Use a regular expression to extract the repository name and file path from the URL
    let re = Regex::new(r"github\.com/([^/]+)/([^/]+)/blob/([^/]+)/(.*)").unwrap();
    let caps = re.captures(url).ok_or_else(|| std::io::Error::new(
        std::io::ErrorKind::Other,
        "Failed to parse GitHub URL",
    ))?;

    let user = caps.get(1).unwrap().as_str();
    let repo = caps.get(2).unwrap().as_str();
    let branch = caps.get(3).unwrap().as_str();
    let file_path = caps.get(4).unwrap().as_str();

    let api_url = format!("https://api.github.com/repos/{}/{}/contents/{}?ref={}", user, repo, file_path, branch);
    let client = reqwest::Client::new();
    let request = client.get(&api_url)
                        .header("Authorization", format!("token {}", token));

    let response = request.send().await?;
    let status = response.status();

    if !status.is_success() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to fetch: {}", status),
        )));
    }

    let content = response.text().await?;
    let json: Value = serde_json::from_str(&content)?;

    println!("JSON response: {:?}", json);

    Ok(())
}