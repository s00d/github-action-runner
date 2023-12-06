use git2::Repository;
use regex::Regex;

pub(crate) fn get_git_owner() -> Result<String, Box<dyn std::error::Error>> {
    let repo = Repository::open(".")?;
    let origin = repo.find_remote("origin")?;
    let url = origin.url().ok_or("No URL found for origin")?;
    let re = Regex::new(r"github\.com[/:](.*?)/")?;
    let caps = re.captures(url).ok_or("No match found in URL")?;
    Ok(caps.get(1).map_or("", |m| m.as_str()).to_string())
}

pub(crate) fn get_git_repo() -> Result<String, Box<dyn std::error::Error>> {
    let repo = Repository::open(".")?;
    let origin = repo.find_remote("origin")?;
    let url = origin.url().ok_or("No URL found for origin")?;
    let re = Regex::new(r"github\.com[/:].*?/(.*?)(\.git)?$")?;
    let caps = re.captures(url).ok_or("No match found in URL")?;
    Ok(caps.get(1).map_or("", |m| m.as_str()).to_string())
}

pub(crate) fn get_git_tree_name() -> Result<String, Box<dyn std::error::Error>> {
    let repo = Repository::open(".")?;
    let head = repo.head()?;
    let name = head.shorthand().ok_or("No shorthand found for head")?;
    Ok(name.to_string())
}