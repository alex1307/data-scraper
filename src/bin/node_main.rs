use data_scraper::{utils::configure_log4rs, services::node::{start_searches, print_stream}};

#[tokio::main]
async fn main(){
    configure_log4rs("config/loggers/dev_log4rs.yml");
    let (tx, mut rx) = crossbeam::channel::unbounded::<String>();
    let task = tokio::spawn(async move {
        start_searches(tx).await;
    });
    let task2 = tokio::spawn(async move {
        print_stream(&mut rx).await;
    });
    tokio::join!(task, task2);
}