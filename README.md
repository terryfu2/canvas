# canvas

poc of frontend with pixel grid, ability to add and remove color on pixels




### To-dos
```
pixel select popup css positoning scuffed when clicked neaar edge
client side lock window when pixel changed
```

## Proxy
Run before running npm start on frontend
```
cd proxy/src
node proxy.js
```
## Backend
```
Download docker https://www.docker.com/products/docker-desktop/ (windows)
# Create Container
docker compose up -d
# Stop
docker compose down
When launching for the first time you will have to wait for the backend to compile then refresh the frontend. It may take a while (like 15 min on my machine).
```
## Run without docker
```
Follow README.md in backend then frontend
```