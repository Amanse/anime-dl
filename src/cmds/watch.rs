use anyhow::{anyhow, bail, Result};
use clap::ArgMatches;
use reqwest_middleware::ClientWithMiddleware;

use crate::providers;
use crate::types::{SearchResult, StreamLink};
use crate::utils::search_results_to_table;

pub async fn command(client: &ClientWithMiddleware, args: &ArgMatches<'_>) -> Result<()> {
    let provider = args.value_of("provider").unwrap();
    let choice = args
        .value_of("choice")
        .unwrap_or("-1")
        .parse::<i32>()
        .unwrap();
    let query = args.value_of("query").unwrap();
    let ep_range = args.value_of("episode").unwrap_or("1:");

    let search_results = providers::search(client, provider, query).await?;
    let chosen = crate::utils::user_select_result(search_results, choice)?;

    let episodes = providers::get_episodes(client, provider, chosen.url.as_str()).await?;

    let ep_range = crate::utils::parse_episode_range(
        ep_range,
        episodes.iter().map(|x| x.ep_num).max().unwrap_or(1),
    );

    let episodes = episodes
        .iter()
        .filter(|x| ep_range.contains(&x.ep_num))
        .collect::<Vec<_>>();

    // TODO: We should not gather all the episodes at once
    let mut streams: Vec<Vec<StreamLink>> = Vec::new();
    for episode in episodes {
        let Ok(streams_) = providers::get_streams(client, provider, episode.url.as_str()).await else {
            continue;
        };

        streams.push(streams_);
    }

    println!("Streams: {:#?}", streams);

    Ok(())
}
