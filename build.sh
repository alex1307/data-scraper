cargo build --release
cp target/release/crawler ~/Software/docker-env/crawler 
cargo  build --features kafka --release
cp target/release/crawler ~/Software/docker-env/crawler/puppeteer
 