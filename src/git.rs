use git2::Repository;
use regex::Regex;

pub(crate) fn get_git_owner() -> Result<String, git2::Error> {
    let repo = Repository::open(".")?;
    let origin = repo.find_remote("origin")?;
    let url = origin.url().unwrap();
    let re = Regex::new(r"github\.com[/:](.*?)/").unwrap();
    let caps = re.captures(url).unwrap();
    Ok(caps.get(1).map_or("", |m| m.as_str()).to_string())
}

pub(crate) fn get_git_repo() -> Result<String, git2::Error> {
    let repo = Repository::open(".")?;
    let origin = repo.find_remote("origin")?;
    let url = origin.url().unwrap();
    let re = Regex::new(r"github\.com[/:].*?/(.*?)(\.git)?$").unwrap();
    let caps = re.captures(url).unwrap();
    Ok(caps.get(1).map_or("", |m| m.as_str()).to_string())
}

pub(crate) fn get_git_tree_name() -> Result<String, git2::Error> {
    let repo = Repository::open(".")?;
    let head = repo.head()?;
    let name = head.shorthand().unwrap();
    Ok(name.to_string())
}