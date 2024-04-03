## Setup

Run the primary proxy first
```
cd proxy
npm i
cd src
node proxy.js
```

Next run the backup proxy (in a different terminal/machine)
```
node backup_proxy.js
```