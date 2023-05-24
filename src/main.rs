use octocrab::models::pulls::PullRequest;
use octocrab::params::pulls::Sort;
use octocrab::params::{Direction, State};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = Opts {
        repo: "quinn-rs/quinn".to_owned(),
        target: "main".to_owned(),
        end: 1435,
        team: vec!["djc".to_owned(), "Ralith".to_owned()],
    };

    let (org, repo) = match opts.repo.split_once('/') {
        Some(parts) => parts,
        None => return Err(anyhow::anyhow!("invalid repo name: {:?}", opts.repo)),
    };

    let client = octocrab::instance();
    let mut page = client
        .pulls(org, repo)
        .list()
        .state(State::Closed)
        .base(&opts.target)
        .sort(Sort::Updated)
        .direction(Direction::Descending)
        .per_page(100)
        .send()
        .await?;

    let mut closed = Vec::new();
    'outer: loop {
        for pr in &page {
            if pr.number == opts.end {
                break 'outer;
            } else if pr.merged_at.is_none() {
                closed.push(pr.clone());
                continue;
            }

            let author = pr.user.as_ref().and_then(|author| {
                if author.login.starts_with("dependabot") {
                    return None;
                }

                match opts.team.contains(&author.login) {
                    false => Some(&author.login),
                    true => None,
                }
            });

            match author {
                Some(author) => println!(
                    "* {} (#{}, thanks to @{})",
                    pr.title.as_deref().unwrap(),
                    pr.number,
                    author
                ),
                None => println!("* {} (#{})", pr.title.as_deref().unwrap(), pr.number),
            };
        }

        page = match client.get_page::<PullRequest>(&page.next).await? {
            Some(page) => page,
            None => break,
        };
    }

    println!("\n\n** CLOSED **\n\n");
    for pr in closed {
        println!("* {} (#{})", pr.title.as_deref().unwrap(), pr.number);
        println!("  {}\n", pr.html_url.as_ref().unwrap());
    }

    Ok(())
}

struct Opts {
    repo: String,
    target: String,
    end: u64,
    team: Vec<String>,
}
