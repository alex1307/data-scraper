#!/bin/bash

# Define a User-Agent string
USER_AGENT="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36"

# Initial GET request to car.gr with User-Agent
curl -A "$USER_AGENT" -o "initial_page.html" "https://car.gr"

# Second GET request to the specific cars search page with User-Agent
curl -A "$USER_AGENT" -o "cars_search_page.html" "https://www.car.gr/quick-search/vehicles/cars?category=15001&root=15000"

# Loop from 1 to 100
for i in {1..100}
do
    # Run the curl command with User-Agent and check if the response contains '429'
    response=$(curl -s -A "$USER_AGENT" -w "%{http_code}" -o "$i.html" "curl "https://app.scrapingbee.com/api/v1/?api_key=3WRJGHKRYTI073VHGRB61COHG5D8U77IIIGSDC53Y9BKC25BXZTW4W773GYL4JOSKNNP2OEKWSEN75O5&url=https://www.car.gr/classifieds/cars/?category=15001&pg="$i"&lang=en")
    http_code=$(tail -n1 <<< "$response") # Extract the HTTP code from the response
    echo "Page $i: $http_code"
    if [ "$http_code" -eq 429 ]; then
        echo "Rate limit hit, waiting longer..."
        sleep 60 # Wait for 60 seconds before retrying
        ((i--)) # Decrement 'i' to retry the same iteration
        # Initial GET request to car.gr with User-Agent
        curl -A "$USER_AGENT" -o "initial_page.html" "https://car.gr"

        # Second GET request to the specific cars search page with User-Agent
        curl -A "$USER_AGENT" -o "cars_search_page.html" "https://www.car.gr/quick-search/vehicles/cars?category=15001&root=15000"

        continue
    fi

    # Wait for a specified time (e.g., 10 seconds) before making the next request
    sleep 10
done
