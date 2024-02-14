#!/bin/bash
cp config/schedulers/*.plist ~/Library/LaunchAgents/
launchctl list | grep com.ayagasha
launchctl unload ~/Library/LaunchAgents/details.plist
launchctl unload ~/Library/LaunchAgents/listing.plist
launchctl unload ~/Library/LaunchAgents/details.plist

launchctl load ~/Library/LaunchAgents/details.plist
launchctl load ~/Library/LaunchAgents/listing.plist
launchctl load ~/Library/LaunchAgents/details.plist

launchctl remove com.ayagasha.scraper.details

id -u

launchctl kickstart -k gui/501/com.ayagasha.scraper.metadata

grep -rHIin [Search text]  ./src
#filter
grep -rHIin [Search text]  ./src/main/ | grep -v node_modules

cargo clippy --fix --bin "crawler" --allow-staged --allow-dirty