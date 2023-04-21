use std::{thread, vec};

use data_scraper::{
    configure_log4rs,
    mobile_scraper::{
        details_scraper::read_list,
        model::{Message, MobileDetails, Processor},
    },
};

fn main() {
    configure_log4rs();

    let (tx, rx) = crossbeam::channel::unbounded::<Message<MobileDetails>>();
    let mut processor: Processor<MobileDetails> =
        Processor::new(rx, "resources/data/mobile_details.csv");
    let mut tasks = vec![];
    let links = vec!["s4g00w".to_string()];
    tasks.push(thread::spawn(move || processor.handle()));
    tasks.push(thread::spawn(move || {
        read_list(
            "resources/data/mobile_sold_1.csv",
            links.clone(),
            &tx.clone(),
        )
    }));
    for task in tasks {
        task.join().unwrap();
    }
}