#!/bin/bash

# https://developers.google.com/youtube/registering_an_application
YT_API_KEY=""

## Get Captions
## > Zu dem Video: https://www.youtube.com/watch?v=KhPQtXQpiZc
#curl "https://www.googleapis.com/youtube/v3/captions?part=snippet&videoId=KhPQtXQpiZc&key=${YT_API_KEY}"

CAPTION_ID=""
CLIENT_ID=""
CLIENT_SECRET=""

# token
curl --request POST \
  --url "https://oauth2.googleapis.com/token" \
  --header "Content-Type: application/x-www-form-urlencoded" \
  --data-urlencode "client_id=${CLIENT_ID}" \
  --data-urlencode "client_secret=${CLIENT_SECRET}" \
  --data-urlencode "refresh_token=YOUR_REFRESH_TOKEN" \
  --data-urlencode "grant_type=refresh_token"


## Download
curl -H "Authorization: Bearer ACCESS_TOKEN" \
     "https://www.googleapis.com/youtube/v3/captions/${CAPTION_ID}?tfmt=ttml"

