<h1 align="center">
  Canvas
</h1>

<p align="center">
A distributed recreation of [r/place](https://en.wikipedia.org/wiki/R/place)
</p>

![plot](./imgs/demo.PNG)

## Architecture  
![plot](./imgs/arch.PNG)


## Features
- Replicated Proxy and Backend Processes
- Detection of Failure and auto reconnection to processes when brought back up
- Live Updates across Clients

## Setup

### Frontend
Follow the instructions in [`the frontend readme`](frontend/README.md)

### Proxy
*Run before running npm start on frontend*<br>
Follow the instructions in [`the proxy readme`](proxy/README.md)

### Backend
Follow the instructions in [`the backend readme`](backend/README.md)

#### Setup csv tables
truncate canvas;
https://www.postgresqltutorial.com/postgresql-tutorial/import-csv-file-into-posgresql-table/

