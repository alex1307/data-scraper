#!/bin/bash
cp config/schedulers/*.plist ~/Library/LaunchAgents/
launchctl list | grep com.ayagasha
launchctl unload ~/Library/LaunchAgents/details.plist
launchctl unload ~/Library/LaunchAgents/listing.plist
launchctl unload ~/Library/LaunchAgents/details.plist

launchctl load ~/Library/LaunchAgents/details.plist
launchctl load ~/Library/LaunchAgents/listing.plist
launchctl load ~/Library/LaunchAgents/details.plist

id -u

launchctl kickstart -k gui/501/com.ayagasha.scraper.metadata