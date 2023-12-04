#!/bin/bash

# ScrapingBee API key and base URL
API_KEY="3WRJGHKRYTI073VHGRB61COHG5D8U77IIIGSDC53Y9BKC25BXZTW4W773GYL4JOSKNNP2OEKWSEN75O5"
BASE_URL="https://app.scrapingbee.com/api/v1/"

# The URL you want to scrape
TARGET_URL="https://www.car.gr/classifieds/cars/?category=15001&pg="

# Loop from 1 to 100
for i in {1..100}
do
    # Construct the full ScrapingBee URL
    TARGET_URL="https%3A%2F%2Fwww.car.gr%2Fclassifieds%2Fcars%2F%3Fcategory%3D15001%26lang%3Den%26pg%3D"${i}
    echo $TARGET_URL
    FULL_URL="${BASE_URL}?api_key=${API_KEY}&url=${TARGET_URL}"

    # Use curl to make the request
    curl -o "$i.html" "$FULL_URL"

    # Wait for a specified time before making the next request
    sleep 10
done
