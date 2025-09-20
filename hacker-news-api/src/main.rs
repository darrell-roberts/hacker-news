// use hacker_news_api::subscribe_top_stories;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    // let (mut rx, _handle) = subscribe_top_stories();

    // let mut old_keys = Vec::new();

    // while let Some(mut data) = rx.recv().await {
    //     data.data.sort_unstable();
    //     // println!("received data: {data:?}");

    //     let new_keys = data
    //         .data
    //         .iter()
    //         .copied()
    //         .take(50)
    //         .filter(|id| old_keys.binary_search(id).is_err())
    //         .collect::<Vec<_>>();

    //     println!("new keys total {} {new_keys:?}", new_keys.len());

    //     old_keys = data.data;
    // }

    Ok(())
}
