#!/bin/bash

# User-Agent and other headers
USER_AGENT="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36"
ACCEPT="application/x-clarity-gzip"
ACCEPT_ENCODING="gzip, deflate, br"
ACCEPT_LANGUAGE="en-US,en;q=0.9,bg;q=0.8"
DNT="1"
REFERER="https://www.car.gr/classifieds/cars/?category=15001&condition=new&offer_type=sale&pg=1"

# Cookie jar file
COOKIE_JAR="cookies.txt"

# Initial GET request to car.gr with User-Agent and cookie handling
curl -A "$USER_AGENT" \
     -H "Accept: $ACCEPT" \
     -H "Accept-Encoding: $ACCEPT_ENCODING" \
     -H "Accept-Language: $ACCEPT_LANGUAGE" \
     -H "Dnt: $DNT" \
     -H "Referer: $REFERER" \
     -c $COOKIE_JAR \
     -o "initial_page.html" "https://car.gr"

# Second GET request to the specific cars search page with User-Agent and cookie handling
curl -A "$USER_AGENT" \
     -H "Accept: $ACCEPT" \
     -H "Accept-Encoding: $ACCEPT_ENCODING" \
     -H "Accept-Language: $ACCEPT_LANGUAGE" \
     -H "Dnt: $DNT" \
     -H "Referer: $REFERER" \
     -b $COOKIE_JAR -c $COOKIE_JAR \
     -o "cars_search_page.html" "https://www.car.gr/quick-search/vehicles/cars?category=15001&root=15000"

# Loop from 1 to 100
for i in {1..100}
do
    # Run the curl command with User-Agent, additional headers, cookie handling, and check for '429'
    response=$(curl -s -A "$USER_AGENT" \
                     -H "Accept: $ACCEPT" \
                     -H "Accept-Encoding: $ACCEPT_ENCODING" \
                     -H "Accept-Language: $ACCEPT_LANGUAGE" \
                     -H "Dnt: $DNT" \
                     -H "Referer: $REFERER" \
                     -b $COOKIE_JAR -c $COOKIE_JAR \
                     -w "%{http_code}" -o "$i.html" \
                     "https://www.car.gr/classifieds/cars/?category=15001&pg=$i&lang=en")
    http_code=$(tail -n1 <<< "$response") # Extract the HTTP code from the response

    if [ "$http_code" -eq 429 ]; then
        echo "Rate limit hit, waiting longer..."
        sleep 60 # Wait for 60 seconds before retrying
        ((i--)) # Decrement 'i' to retry the same iteration
        continue
    fi

    # Wait for a specified time (e.g., 10 seconds) before making the next request
    sleep 10
done
