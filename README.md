How to use:
- grab the gecko driver: https://github.com/mozilla/geckodriver/releases
- ensure firefox is installed: https://www.mozilla.org/en-US/firefox/new/
- run the program once to generate config.json. It will exit early, but give you a template file
- fill out the firefox executable path & the gecko driver path in the json
- fill out the jwt field with the cookie value from duo's site (inpect element -> storage -> cookies -> duolingo.com -> jwt_token)

If using ja, es, or de, the generated fallbacks should be enough. You can add more, following the generated ones as examples. 
For faster parsing or special use cases, copy the words list from https://www.duolingo.com/practice-hub/words and paste them into words.txt.
